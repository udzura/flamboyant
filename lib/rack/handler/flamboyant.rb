require 'flamboyant'
require 'webrick'
require 'webrick/httprequest'
require 'rack'
require 'uri'
require 'stringio'

class WEBrick::HTTPRequest
  def generate_from_string(reqline, lines)
    @peeraddr = [] # TODO set in Rust
    @addr = []
    
    read_request_one_line(reqline)
    if @http_version.major > 0
      read_header_str(lines)
      @header['cookie'].each{|cookie|
        @cookies += WEBrick::Cookie::parse(cookie)
      }
      @accept = WEBrick::HTTPUtils.parse_qvalues(self['accept'])
      @accept_charset = WEBrick::HTTPUtils.parse_qvalues(self['accept-charset'])
      @accept_encoding = WEBrick::HTTPUtils.parse_qvalues(self['accept-encoding'])
      @accept_language = WEBrick::HTTPUtils.parse_qvalues(self['accept-language'])
    end
    return if @request_method == "CONNECT"
    return if @unparsed_uri == "*"

    begin
      setup_forwarded_info
      @request_uri = parse_uri(@unparsed_uri)
      @path = WEBrick::HTTPUtils::unescape(@request_uri.path)
      @path = WEBrick::HTTPUtils::normalize_path(@path)
      @host = @request_uri.host
      @port = @request_uri.port
      @query_string = @request_uri.query
      @script_name = ""
      @path_info = @path.dup
    rescue
      raise WEBrick::HTTPStatus::BadRequest, "bad URI `#{@unparsed_uri}'."
    end

    if /\Aclose\z/io =~ self["connection"]
      @keep_alive = false
    elsif /\Akeep-alive\z/io =~ self["connection"]
      @keep_alive = true
    elsif @http_version < "1.1"
      @keep_alive = false
    else
      @keep_alive = true
    end
  end    

  def read_request_one_line(line)
    @request_line = line
    @request_bytes = @request_line.bytesize
    if @request_bytes >= WEBrick::HTTPRequest::MAX_URI_LENGTH and @request_line[-1, 1] != LF
      raise WEBrick::HTTPStatus::RequestURITooLarge
    end

    @request_time = Time.now
    if /^(\S+)\s+(\S++)(?:\s+HTTP\/(\d+\.\d+))?\r?\n/mo =~ @request_line
      @request_method = $1
      @unparsed_uri   = $2
      @http_version   = WEBrick::HTTPVersion.new($3 ? $3 : "0.9")
    else
      rl = @request_line.sub(/\x0d?\x0a\z/o, '')
      raise WEBrick::HTTPStatus::BadRequest, "bad Request-Line `#{rl}'."
    end
  end

  def read_header_str(lines)
    lines.each do |line|
      break if /\A(\r\n|\n)\z/om =~ line
      if (@request_bytes += line.bytesize) > WEBrick::HTTPRequest::MAX_URI_LENGTH
        raise WEBrick::HTTPStatus::RequestEntityTooLarge, 'headers too large'
      end
      @raw_header << line
    end
    @header = WEBrick::HTTPUtils::parse_header(@raw_header.join)
  end
end

module Rack
  module Handler
    class Flamboyant
      SERVER_NAME = "Flamboyant/0.1.0 experimental"
      
      def self.run(app, **options)
        ENV["PORT"] ||= options[:Port]&.to_s || "9292"
        
        server = ::Flamboyant.new
        config = ::WEBrick::Config::HTTP.merge(
          ServerSoftware: SERVER_NAME
        )
        server.serve(
          lambda { |req|
            begin
              head, rbody = req.split("\r\n\r\n")
              heads = head.lines
              wreq = ::WEBrick::HTTPRequest.new(config)
              wreq.generate_from_string(heads[0], heads[1..heads.size])
              
              env = wreq.meta_vars
              env.delete_if { |k, v| v.nil? }
              rack_input = StringIO.new(rbody || "")
              rack_input.set_encoding(Encoding::BINARY)

              env.update(
                ::Rack::RACK_VERSION      => ::Rack::VERSION,
                ::Rack::RACK_INPUT        => rack_input,
                ::Rack::RACK_ERRORS       => $stderr,
                ::Rack::RACK_URL_SCHEME   => ["yes", "on", "1"].include?(env[::Rack::HTTPS]) ? "https" : "http",
                ::Rack::RACK_IS_HIJACK    => true,
                ::Rack::RACK_HIJACK       => lambda { raise NotImplementedError, "only partial hijack is supported."},
                ::Rack::RACK_HIJACK_IO    => nil,
                'rack.multithread'        => false,
                'rack.multiprocess'       => false,
                'rack.run_once'           => false
              )

              status, headers, body = app.call(env)
              headers["Server"] = SERVER_NAME
              res = [
                "HTTP/1.1 #{status} OK",
                *headers.map{|k, v| "#{k}: #{v}"},
                "",
                body.join
              ].join("\r\n")
              
              return res
            rescue => e
              body = e.inspect + e.backtrace.join("\n")
              
              res = [
                "HTTP/1.1 503 Internal Server Error",
                "Content-Type: text/plain",
                "Content-Length: #{body.size}",
                "",
                body
              ].join("\r\n")
              return res
            end
          }
        )
      end
    end

    register 'flamboyant', Flamboyant
  end
end
