use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::*;

#[test]
fn attest_test() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        // Regular attest works for all valid confidence
        for i in 0..11 {
            assert_ok!(SocialGraph::attest(Origin::signed(i+1), i, i.try_into().unwrap()));
        }
        // Test invalid confidence
        assert_noop!(SocialGraph::attest(Origin::signed(2), 1, 11), Error::<Test>::ConfidenceOutOfBounds);  
        // Test invalid target
        assert_noop!(SocialGraph::attest(Origin::signed(2), 2, 1), Error::<Test>::SelfAttestationError);   
        println!("{:?}", <AccountData<Test>>::get(3));
    });
}

#[test]
fn challenge_and_vote_test() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(SocialGraph::challenge(Origin::signed(1), 2));
    });
}
