app = lambda {|_env|
  return [200, {"Content-Type" => "text/plain"}, ["Hello rack app"]]
}

run app
