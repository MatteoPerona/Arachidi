use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn attest_test() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(SocialGraph::attest(Origin::signed(1), 3, 1));
        assert_noop!(SocialGraph::attest(Origin::signed(2), 2, 1), Error::<T>::SelfAttestationError);
        dbg!("{:?}", Error::ConfidenceOutOfBounds)
    });
}

