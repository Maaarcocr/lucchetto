require_relative "./lib/simple"

t = Thread.new do
  puts slow_func_no_lock("hello")
end

1..10.times do
  sleep 0.1
  puts "main thread"
end

t.join
