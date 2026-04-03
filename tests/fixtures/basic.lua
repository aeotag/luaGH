-- tests/fixtures/basic.lua
-- A basic Lua file with no issues for baseline testing.

local function greet(name)
    print("Hello, " .. name .. "!")
end

local function add(a, b)
    return a + b
end

local result = add(1, 2)
greet("world")
print(result)
