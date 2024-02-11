# Todo List

- [x] Remove hover effect for lines without any command
- [x] Rename include to import
- [x] Alias #- to #-pre since most commands will be prefixes
- [x] Add separators besides empty lines such as mid view titles
- [x] Add mid list command cleanup allowing multiple commands per file without include
- [ ] Add a way to require a binary in path or at an exact location, otherwise disabling command
- [ ] Add warnings for wrong syntax in directives, missing binaries or missing imports
- [ ] Add a way to disable warnings for the next directive
- [x] Rewrite all line add code to use ParsedLine
- [x] Move skip_directives from bool to reader, edit: to map
- [x] Add command map
- [ ] Rewrite cmd to separate args and command, and require it be finished to use
- [ ] Add some values to environment when executing
- [ ] Add lines created by executing a command
- [ ] Add lines depending on success of next or previous line creation
- [ ] Simplify creation and storage of commands and lines to eliminate locks
- [x] Merge Directive and ParsedLine
