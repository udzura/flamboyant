require 'fileutils'
lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "flamboyant/version"

Gem::Specification.new do |spec|
  spec.name          = "flamboyant"
  spec.version       = Flamboyant::VERSION
  spec.authors       = ["Uchio Kondo"]
  spec.email         = ["udzura@udzura.jp"]

  spec.summary       = %q{Experimental web server for Ruby, written as a Rust-made gem}
  spec.description   = %q{Experimental web server for Ruby, written as a Rust-made gem}
  spec.homepage      = "https://github.com/udzura/flamboyant"
  spec.license = "MIT"
  
  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files = Dir.chdir(__dir__) do
    `git ls-files -z`.split("\x0").reject do |f|
      (f == __FILE__) || \
      f.match(%r{\A(?:(?:bin|test|spec|features)/|\.(?:git|travis|circleci)|appveyor)})
    end + ["Cargo.lock"]
  end
  # spec.bindir        = "exe"
  # spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]
  spec.extensions    = ["Cargo.toml"]

  spec.add_dependency "rack"
end
