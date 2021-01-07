use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
        assert_eq!(
            Proofs::<Test>::get(&claim),
            (1, frame_system::Module::<Test>::block_number())
        )
    })
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyClaimed
        );
    })
}

#[test]
fn revoke_claim_workes() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
    })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];

        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::NoSuchProof
        );
    })
}

#[test]
fn transfer_claim_workes() {
    new_test_ext().execute_with(|| {
        let claim = vec![0, 1];
        let dest_address: u64 = 111; // 转移的目标用户的AccountId
        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

        // 确认目前属于AccountId为1的人
        assert_eq!(
            Proofs::<Test>::get(&claim),
            (1, frame_system::Module::<Test>::block_number())
        );

        // 执行转移存证的操作
        assert_ok!(PoeModule::transfer_claim(
            Origin::signed(1),
            dest_address,
            claim.clone()
        ));

        // 判断现在这个存证是否属于AccountId=111的人
        assert_eq!(
            Proofs::<Test>::get(&claim),
            (111, frame_system::Module::<Test>::block_number())
        );
    })
}
