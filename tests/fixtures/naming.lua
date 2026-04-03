-- tests/fixtures/naming.lua
-- Naming convention test cases: various violations and correct usage.

-- CORRECT: snake_case local
local my_variable = 10

-- VIOLATION: PascalCase local variable (should be snake_case)
local MyBadVariable = 20

-- CORRECT: snake_case function
local function compute_sum(a, b)
    return a + b
end

-- VIOLATION: uppercase function name (should be snake_case)
local function ComputeProduct(a, b)
    return a * b
end

-- CORRECT: UPPER_CASE global
MY_CONSTANT = 42

-- VIOLATION: lowercase global (should be UPPER_CASE)
my_global = 99

-- CORRECT: metamethods should be ignored
local mt = {}
function mt:__tostring()
    return "example"
end

function mt:__index(key)
    return nil
end

-- CORRECT: underscore prefix should be ignored
local _unused = "ignore me"

-- CORRECT: snake_case method
local obj = {}
function obj:get_value()
    return self.value
end

-- VIOLATION: PascalCase method
function obj:SetValue(v)
    self.value = v
end

-- Use the variables to avoid unused warnings
print(my_variable, MyBadVariable, MY_CONSTANT, my_global, _unused)
compute_sum(1, 2)
ComputeProduct(3, 4)
obj:get_value()
obj:SetValue(5)
