# Arachidi 🥜
Final Project for Polkadot Blockchain Academy 2022

# Overview
I created a social graph pallet @ `substrate-node-template/pallets/social-graph`.
My aim was to create a decentralized peer identity system that relies on a blockchain to preserve data integrity.

Each "valid" account can attest for any other with a confidence value between 0..10 (inclusive). 
The confidence value represents how sure the attester is that the account they're validating is the unique account linked with someone they know in the real world.
An account is deemed "valid" (allowed to attest, challenge, and vote) when their # attestations is greater than the network average, their average confidence is greater than the average of the entire network, and they are not banned.
In the future I would like to add a minimum number of blocks since an account's birth block to be considered "valid".

As I hinted before, each "valid" account can also challnge other accounts' integrity (essentially opening a vote that stays open for n blocks) and voto on open challenges.
This mechanism is intended to provide a rudamentary form of sybil resistance. In the future I would like to add a storage map for flagging accounts that have validated banned accounts. 
My hope is that the only way for fake accounts to proliferate is for them to attest to each other's existence creating unstable networks that can be taken down efficiently. 

I also need to implement a system so that users cannot spam the network with challenges, and a method to punish users that challenge too many accounts that are deemed real. 

# Considerations 
The project is very unfinished. Tests do not currently run because I am still testing the voting system I made from scratch. 
