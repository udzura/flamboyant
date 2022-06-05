require 'flamboyant'
require 'rack'
module Rack
  module Handler
    class Flamboyant
      def self.run(app, **options)
        server = ::Flamboyant.new
        server.serve(
          lambda { |req|
            env = {}
            env["CONTENT_TYPE"] = "text/plain"
            env[RACK_ERRORS] = $stderr

            status, headers, body = app.call(env)
            res = [
              "HTTP/1.1 #{status} OK",
              *headers.map{|k, v| "#{k}: #{v}"},
              "",
              body.join
            ].join("\r\n")
            
            return res
          })
      end
    end

    register 'flamboyant', Flamboyant
  end
end
