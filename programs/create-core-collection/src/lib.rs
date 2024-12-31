use anchor_lang::prelude::*;
use mpl_core::{
    accounts::BaseCollectionV1,
    instructions::{CreateCollectionV2CpiBuilder, CreateV2CpiBuilder, TransferV1Cpi},
    types::{
        ExternalCheckResult, ExternalPluginAdapterInitInfo, HookableLifecycleEvent, OracleInitInfo,
        ValidationResultsOffset,
    },
    ID as MPL_CORE_ID,
};

declare_id!("ER9AadmM55TVTFQGz8YDS94pYwpMDD3BEMSHsRXxpj92");

#[program]
pub mod create_core_collection {

    use super::*;

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        args: CreateCollectionArgs,
    ) -> Result<()> {
        let update_authority = match &ctx.accounts.update_authority {
            Some(update_authority) => Some(update_authority.to_account_info()),
            None => None,
        };

        let onchain_oracle_plugin = pubkey!("AwPRxL5f6GDVajyE1bBcfSWdQT58nWMoS36A1uFtpCZY");
        let oracle_init_info = OracleInitInfo {
            base_address: onchain_oracle_plugin,
            init_plugin_authority: None,
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                // flag 4 - can only reject. ref: https://github.com/metaplex-foundation/mpl-core/blob/87e2e354fcba8e8f3c9a1f52d002892986266ba8/programs/mpl-core/src/plugins/lifecycle.rs#L46
                ExternalCheckResult { flags: 4 },
            )],
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Anchor),
        };
        let external_plugin_adapters: Vec<ExternalPluginAdapterInitInfo> =
            vec![ExternalPluginAdapterInitInfo::Oracle(oracle_init_info)];

        CreateCollectionV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .collection(&ctx.accounts.collection.to_account_info())
            .payer(&ctx.accounts.payer.to_account_info())
            .update_authority(update_authority.as_ref())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .external_plugin_adapters(external_plugin_adapters)
            .invoke()?;

        ctx.accounts.collection_info.collection_address = ctx.accounts.collection.key();
        ctx.accounts.collection_info.is_created = true;

        Ok(())
    }

    pub fn create_asset(ctx: Context<CreateAsset>, args: CreateAssetsArgs) -> Result<()> {
        let is_created = &ctx.accounts.collection_info.is_created;
        require!(is_created, CustomError::CollectionIsNotCreated);

        let collection_address = &ctx.accounts.collection_info.collection_address;
        let collection = &ctx.accounts.collection.to_account_info();
        if collection_address.as_ref() != collection.key().as_ref() {
            msg!("Wrong Collection");
            return Err(error!(CustomError::WrongCollection));
        }

        let authority = match &ctx.accounts.authority {
            Some(authority) => Some(authority.to_account_info()),
            None => None,
        };

        let owner = match &ctx.accounts.owner {
            Some(owner) => Some(owner.to_account_info()),
            None => None,
        };

        let update_authority = match &ctx.accounts.update_authority {
            Some(update_authority) => Some(update_authority.to_account_info()),
            None => None,
        };

        CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
            .asset(&ctx.accounts.asset.to_account_info())
            .collection(Some(collection.as_ref()))
            .authority(authority.as_ref())
            .payer(&ctx.accounts.payer.to_account_info())
            .owner(owner.as_ref())
            .update_authority(update_authority.as_ref())
            .system_program(&ctx.accounts.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .invoke()?;
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>) -> Result<()> {
        TransferV1Cpi {
            asset: &ctx.accounts.asset.to_account_info(),
            collection: ctx.accounts.collection.as_ref(),
            payer: &ctx.accounts.payer.to_account_info(),
            authority: ctx.accounts.authority.as_deref(),
            new_owner: &ctx.accounts.new_owner.to_account_info(),
            system_program: ctx.accounts.system_program.as_ref(),
            log_wrapper: ctx.accounts.log_wrapper.as_ref(),
            __program: &ctx.accounts.mpl_core,
            __args: mpl_core::instructions::TransferV1InstructionArgs {
                compression_proof: None,
            },
        }
        .invoke()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub collection: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + CollectionInfo::INIT_SPACE,
        seeds = [b"collectionInfo"],
        bump
    )]
    pub collection_info: Account<'info, CollectionInfo>,
    /// CHECK: this account will be checked by the mpl_core program
    pub update_authority: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CreateAsset<'info> {
    #[account(mut)]
    pub asset: Signer<'info>,
    pub authority: Option<Signer<'info>>,
    #[account(mut)]
    pub collection: Account<'info, BaseCollectionV1>,
    #[account(
        mut,
        seeds = [b"collectionInfo"],
        bump
    )]
    pub collection_info: Account<'info, CollectionInfo>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: this account will be checked by the mpl_core program
    pub owner: Option<UncheckedAccount<'info>>,
    /// CHECK: this account will be checked by the mpl_core program
    pub update_authority: Option<UncheckedAccount<'info>>,
    pub system_program: Program<'info, System>,
    #[account(address = MPL_CORE_ID)]
    /// CHECK: this account is checked by the address constraint
    pub mpl_core_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    /// The address of the asset.
    /// CHECK: Checked in mpl-core.
    #[account(mut)]
    pub asset: AccountInfo<'info>,

    /// The collection to which the asset belongs.
    /// CHECK: Checked in mpl-core.
    #[account(mut)]
    pub collection: Option<AccountInfo<'info>>,

    /// The account paying for the storage fees.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The owner or delegate of the asset.
    pub authority: Option<Signer<'info>>,

    /// The new owner of the asset.
    /// CHECK: Just a destination, no checks needed.
    pub new_owner: AccountInfo<'info>,

    /// The system program.
    /// CHECK: Checked in mpl-core.
    pub system_program: Option<AccountInfo<'info>>,

    /// The SPL Noop program.
    /// CHECK: Checked in mpl-core.
    pub log_wrapper: Option<AccountInfo<'info>>,

    /// The MPL Core program.
    /// CHECK: Checked in mpl-core.
    #[account(address = mpl_core::ID)]
    pub mpl_core: AccountInfo<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct CollectionInfo {
    pub collection_address: Pubkey,
    pub is_created: bool,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateAssetsArgs {
    name: String,
    uri: String,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateCollectionArgs {
    name: String,
    uri: String,
}

#[error_code]
pub enum CustomError {
    #[msg("Collection is not created")]
    CollectionIsNotCreated,
    #[msg("Wrong Collection")]
    WrongCollection,
}
