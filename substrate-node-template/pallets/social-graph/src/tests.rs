use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use crate::*;

#[test]
fn attest_test() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        // Regular attest works for all valid confidence
        for i in 0..11 {
            let account = i;
            // Totals
			match <TotalsCounter<Test>>::try_get() {
				Ok(tup) => println!("tot_attest {} tot_conf {}", tup.0, tup.1),
				Err(_) => println!("Nothing In TotalsCounter"),
			};
			let tot_accounts = <AccountData<Test>>::count();
            println!("tot_accounts {}", tot_accounts);
			// Account
			match <AccountData<Test>>::try_get(account.clone()) {
				Ok(tup) => println!("attest_count {} conf_sum {}", tup.0, tup.1),
				Err(_) => println!("Nothing In Account data for {}", account.clone()),
			};

            assert_ok!(SocialGraph::attest(Origin::signed(i), i+1, i.try_into().unwrap()));
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
        assert_noop!(SocialGraph::challenge(Origin::signed(1), 2), Error::<Test>::ChallengeAlreadyExists);

        // Test voting extremes
        assert_ok!(SocialGraph::vote(Origin::signed(1), 2, -10));
        assert_ok!(SocialGraph::vote(Origin::signed(2), 2, 10));
        assert_ok!(SocialGraph::vote(Origin::signed(3), 2, 0));
        // Failed voting cases
        assert_noop!(SocialGraph::vote(Origin::signed(3), 1, 0), Error::<Test>::ChallengeNotFound);
    });
}
