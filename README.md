# Arachidi 🥜
Final Project for Polkadot Blockchain Academy 2022

# Overview
I created a social graph pallet @ `substrate-node-template/pallets/social-graph`.
My aim was to create a decentralized peer identity system that relies on a blockchain to preserve data integrity.

Each "valid" account can attest for any other with a confidence value between 0..10 (inclusive). 
The confidence value represents how sure the attester is that the account they're validating is the unique account linked with someone they know in the real world.
An account is deemed "valid" (allowed to attest, challenge, and vote) when their # attestations is greater than the network average, their average confidence is greater than the average of the entire network, and they are not banned.
In the future I would like to require a minimum number of blocks since an account's "birth block" to be considered "valid".

As I hinted before, each "valid" account can also challnge other accounts' integrity (essentially opening a vote for n blocks) and vote on open challenges.
This mechanism is intended to provide a rudamentary form of sybil resistance. In the future I would like to add a storage map for flagging accounts that have validated banned accounts to track bad actors. 
My hope is that the only way for fake accounts to proliferate is for them to attest to each other's existence creating unstable networks that can be taken down efficiently. 
I also need to implement a system to prevent users from challenge spamming (staking, limiting challenges per account, ...), and a method to punish users who's challenges are consistetly voted down. 

# Considerations 
While its core functionality works, the project is unfinished. I built the voting system from scratch, and I'm still not 100% sure of the functionality. 
I am also still working on a front end network graph visualization based on: https://codesandbox.io/s/hc064?file=/src/Graph.js

# Conclusions/Future
I hope to build this project from scratch again to reduce complexity and implement prebuilt voting architectures. While I think there are benefits to 
using my homemade voting system, and it was fun to code, I could have spent more time polishing and less time re-inventing the wheel. Going forward, I'm going to test out a few sybil resistance patterns that levarage feeds user data from oracles and/or on-chain sources. Long term, I'd like to build apis that allow developers to automate social transactions. I'd like to create developer tools which enable fully digital company/government infrastructures. Hopefully I can work with parachains that are developing on real world asset ownership on chain to create more usable products. I'd also like to set up a notary ontop of the blockchain as its first userfacing product.  
