mod utils;

#[cfg(test)]
mod create_market {
    use crate::utils::{
        helpers::{airdrop, create_mint, create_token_account},
        setup_functions::{setup_selling_resource, setup_store},
    };
    use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
    use chrono::NaiveDate;
    use solana_program_test::*;

    use mpl_membership_token::{
        accounts as mpl_membership_token_accounts, instruction as mpl_membership_token_instruction,
        state::{Market, MarketState},
        utils::{
            find_treasury_owner_address, puffed_out_string, DESCRIPTION_MAX_LEN, NAME_MAX_LEN,
        },
    };
    use solana_sdk::{
        instruction::Instruction, signature::Keypair, signer::Signer, system_program,
        transaction::Transaction,
    };

    use crate::setup_context;

    #[tokio::test]
    async fn success() {
        setup_context!(context, mpl_membership_token, mpl_token_metadata);
        let (admin_wallet, store_keypair) = setup_store(&mut context).await;

        let (selling_resource_keypair, selling_resource_owner_keypair, _) =
            setup_selling_resource(&mut context, &admin_wallet, &store_keypair).await;

        airdrop(
            &mut context,
            &selling_resource_owner_keypair.pubkey(),
            10_000_000_000,
        )
        .await;

        let market_keypair = Keypair::new();

        let treasury_mint_keypair = Keypair::new();
        create_mint(
            &mut context,
            &treasury_mint_keypair,
            &admin_wallet.pubkey(),
            0,
        )
        .await;

        let (treasury_owner, treasyry_owner_bump) = find_treasury_owner_address(
            &treasury_mint_keypair.pubkey(),
            &selling_resource_keypair.pubkey(),
        );

        let treasury_holder_keypair = Keypair::new();
        create_token_account(
            &mut context,
            &treasury_holder_keypair,
            &treasury_mint_keypair.pubkey(),
            &treasury_owner,
        )
        .await;

        let start_date = NaiveDate::from_ymd(2022, 05, 01)
            .and_hms(00, 00, 00)
            .timestamp() as u64;

        let name = "Marktname".to_string();
        let description = "Marktbeschreibung".to_string();
        let mutable = true;
        let price = 1_000_000;
        let pieces_in_one_wallet = Some(1);

        let accounts = mpl_membership_token_accounts::CreateMarket {
            market: market_keypair.pubkey(),
            store: store_keypair.pubkey(),
            selling_resource_owner: selling_resource_owner_keypair.pubkey(),
            selling_resource: selling_resource_keypair.pubkey(),
            mint: treasury_mint_keypair.pubkey(),
            treasury_holder: treasury_holder_keypair.pubkey(),
            owner: treasury_owner,
            system_program: system_program::id(),
        }
        .to_account_metas(None);

        let data = mpl_membership_token_instruction::CreateMarket {
            _treasyry_owner_bump: treasyry_owner_bump,
            name: name.to_owned(),
            description: description.to_owned(),
            mutable,
            price,
            pieces_in_one_wallet,
            start_date,
            end_date: None,
        }
        .data();

        let instruction = Instruction {
            program_id: mpl_membership_token::id(),
            data,
            accounts,
        };

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &market_keypair,
                &selling_resource_owner_keypair,
            ],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await.unwrap();

        let market_acc = context
            .banks_client
            .get_account(market_keypair.pubkey())
            .await
            .expect("account not found")
            .expect("account empty");

        let market_data = Market::try_deserialize(&mut market_acc.data.as_ref()).unwrap();

        assert_eq!(store_keypair.pubkey(), market_data.store);
        assert_eq!(
            selling_resource_keypair.pubkey(),
            market_data.selling_resource
        );
        assert_eq!(treasury_mint_keypair.pubkey(), market_data.treasury_mint);
        assert_eq!(
            treasury_holder_keypair.pubkey(),
            market_data.treasury_holder
        );
        assert_eq!(treasury_owner, market_data.treasury_owner);
        assert_eq!(selling_resource_owner_keypair.pubkey(), market_data.owner);
        assert_eq!(puffed_out_string(name, NAME_MAX_LEN), market_data.name);
        assert_eq!(
            puffed_out_string(description, DESCRIPTION_MAX_LEN),
            market_data.description
        );
        assert_eq!(mutable, market_data.mutable);
        assert_eq!(price, market_data.price);
        assert_eq!(pieces_in_one_wallet, market_data.pieces_in_one_wallet);
        assert_eq!(MarketState::Created, market_data.state);
    }
}
