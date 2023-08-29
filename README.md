# revm-repro

## Description of the issue
1. Counter contract is deployed on the forked mainnet.
2. We call the Counter contract using different method from another contract.
3. revm doesn't report any issue.
Why? Is there a way to fix that?

## Notes
Most if not all of related logic is it simulate_send.rs, there are no hidden gotchas anywhere.

## To start foundry fork:
cd foundry && npm start

## To start the simulation:
cargo run
