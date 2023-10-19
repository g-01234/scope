# TODO

- redo compile; send cmd + c

- use onDidCloseTerminal for compile
- encode contstructor args w/ copy create code
- other tx args (gasprice, gaslimit)
- better test configs
  - test all
  - gas-report checkbox
  - fork block number
- decode return vals for raw calldata input when possible
  - (parse compiled contracts for matching fn selectors)
- cleaner compilation
- cheatcodes
  - warp/roll
  - etch
  - set nonce
- load from /broadcast/latest
- try check etherscan for ABI?
- second tab
  - initcode generator
  - nice keccak (and friends)
  - vanity generator
- find a way to handle encoding better (diff library?)
  \- leftover types (array, tuple etc)
- better handle paths?
- update on save
- compile shell scripts?
- custom foundry profile

Done

- Imports
- sort fns by mutability
- decode return vals
- fix address copy button
- handle constructor args
- handle reverts
- popups w/ errors
- deploy arbitrary bytecode
- arbitrary calldata
- hex input/output (blocking for bytecode+address args)
  - toggle between hex/dec? - not done
- dropdown for function actions
  - copy raw calldata w/ current inputs
  - copy raw output
- fix debug
- persistence
- dropdown for contract actions
  - storage
  - copy abi
  - copy deployed bytecode
  - copy as interface
  - something w/ events?
  - pyrometer
- fix load address
- accounts (anvil autoimpersonate)
- "the" bug
- get run code from chain not solc output
- cheatcodes
  - set storage
  - set balance
