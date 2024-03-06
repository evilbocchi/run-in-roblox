--
-- Copyright (c) Paradoxum Games 2024
-- This file is licensed under the Mozilla Public License (MPL-2.0). A copy of it is available in the 'LICENSE' file at the root of the repository.
-- This file incorporates changes from rojo-rbx/run-in-roblox, which is licensed under the MIT license.
--
-- Copyright 2019 Lucien Greathouse
-- Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
-- The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
-- THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
--

local fs = require("@lune/fs")
local roblox = require("@lune/roblox")

local plugin = roblox.Instance.new("Script")
plugin.Name = "run-in-roblox-plugin"
plugin.Source = fs.readFile("./plugin.luau")

local logger = roblox.Instance.new("ModuleScript")
logger.Parent = plugin
logger.Name = "Logger"
logger.Source = fs.readFile("./logger.luau")

local model = roblox.serializeModel({plugin})
fs.writeFile("./plugin.rbxm", model)