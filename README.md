A Discord modmail bot written in Rust.

# :warning: THIS IS A WORK IN PROGRESS :warning:
Stability is ***not*** guaranteed yet. Stuff might break at any time.

## Todo
- [x] split big files
  - [x] `bot/modmail.rs`
  - [x] `events.rs`
- [ ] allow modmail to operate in different server than members (might work already, needs testing)
- [ ] time-based things
  - [ ] close in X minutes
  - [ ] block for X minutes
- [x] interactions
  - [x] send messages back to user
  - [x] editing sent messages
  - [x] deleting sent messages
  - [ ] tags
- [ ] somehow allow multiline arguments (might need to use modals)
- [ ] cache in front of database
- [x] allow staff to retrieve message links and other information
