use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};

use std::collections::HashMap;
// This is your program's public key and it will update
// automatically when you build the project.
declare_id!("AH71wvrtpPnLzVYvB2SgLKA6KkNihrsffrKD6TbTCisU");

pub struct NewsArticle {
    pub author: String,
    pub headline: String,
    pub content: String,
}

impl Summary for NewsArticle {
    fn summaryze(&self) {
        msg!("summaryze {}", self.author);
    }
}

pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

pub trait Summary {
    fn summaryze(&self);
}

#[program]
pub mod den {

    use super::*;
    pub fn initialize(ctx: Context<Initialize>, nfts: Vec<Pubkey>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.nfts = nfts;
        Ok(())
    }

    pub fn update_nfts(ctx: Context<UpdateNfts>, nfts: Vec<Pubkey>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.nfts = nfts;
        let mut scores = HashMap::new();

        scores.insert(String::from("Blue"), 10);
        scores.insert(String::from("Blue"), 20);

        let text = "hello world wonderful world";
        let mut map = HashMap::new();
        for word in text.split_whitespace() {
            let count = map.entry(word).or_insert(0);
            *count += 1;
        }
        msg!("{:?}", map);

        Ok(())
    }

    pub fn show_nfts(ctx: Context<ShowNfts>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        for (index, nft_pubkey) in state.nfts.iter().enumerate() {
            msg!("NFT at index {}: {:?}", index, nft_pubkey);
        }

        Ok(())
    }

    pub fn submit_economic_data(
        ctx: Context<SubmitEconomicData>,
        invoice_data: String,
        hsn_number: String,
        amount: u64,
        quantity: u32,
        timestamp: i64,
        signature: String,
    ) -> Result<SubmitResponse> {
        let node = &mut ctx.accounts.node;

        let new_entry = EconomicDataEntry {
            amount,
            quantity,
            timestamp,
            hsn_number: hsn_number.trim().to_string(),
            invoice_data: invoice_data.trim().to_string(),
            signature: signature.trim().to_string(),
            is_verified: false,
        };

        node.data.push(new_entry);

        let mut hasher = Sha256::new();
        hasher.update(invoice_data.as_bytes());
        hasher.update(hsn_number.as_bytes());
        hasher.update(amount.to_le_bytes());
        hasher.update(quantity.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update(signature.as_bytes());
        let transaction_hash = format!("{:x}", hasher.finalize());
       
        Ok(SubmitResponse {
            success: true,
            transaction_hash,
        })
    }


    pub fn validate_invoice_data(
        ctx: Context<ValidateNode>,
        hsn_number: String,
    ) -> Result<()> {
        let node = &mut ctx.accounts.node;
        let admin_pubkey = ctx.accounts.admin.key.to_string();

        // List of admin public keys
        let admin_pubkeys: &[String] = &[
            String::from("FH5uTSXBJF4ZdF6UPPB5hzatuftB7mcyv6zsBWGz488p")
            // Add more admins as needed
        ];

        // Check if the payer's public key is one of the admin public keys
        if !admin_pubkeys.contains(&admin_pubkey) {
            // If the payer is not an admin, return an error
            return Err(ErrorCode::ConstraintSigner.into());
        }

        for entry in node.data.iter_mut() {
            if hsn_number.eq(&entry.hsn_number) {
                entry.is_verified = true;
                node.total_rewards += (entry.invoice_data.len() / 1000) as u64;
                break
            }
        }

        Ok(())
    }

    pub fn validate_node(
        ctx: Context<ValidateNode>,
        _credentials: String,
    ) -> Result<ValidateResponse> {
        let _node = &ctx.accounts.node;

        let is_valid = true;
        let node_status = if is_valid { "active" } else { "inactive" }.to_string();

        Ok(ValidateResponse {
            is_valid,
            node_status,
        })
    }

    pub fn get_node_stats(ctx: Context<GetNodeStats>) -> Result<NodeStatsResponse> {
        let node = &ctx.accounts.node;

        let total_transactions = node.data.len() as u32;
        let total_amount: u64 = node.data.iter().map(|entry| entry.amount).sum();
        let active_since = node.active_since;

        Ok(NodeStatsResponse {
            total_transactions,
            total_amount,
            active_since,
        })
    }

    pub fn remove_node(ctx: Context<RemoveNode>) -> Result<RemoveResponse> {
        let node = &mut ctx.accounts.node;

        node.data.clear();
        node.is_active = false;

        Ok(RemoveResponse {
            status: true,
            message: "Node removed successfully".to_string(),
        })
    }
    pub fn query_economic_data(
        ctx: Context<QueryEconomicData>,
        start_date: i64,
        end_date: i64,
        parameters: QueryParameters,
    ) -> Result<QueryResponse> {
        let node = &ctx.accounts.node;
    
        let data: Vec<EconomicDataEntry> = node
            .data
            .iter()
            .filter(|entry| entry.timestamp >= start_date && entry.timestamp <= end_date)
            .filter(|entry| {
                (parameters.hsn_number.is_empty() || entry.hsn_number == parameters.hsn_number) &&
                (parameters.amount_range.is_none() ||
                 (entry.amount >= parameters.amount_range.unwrap().min &&
                  entry.amount <= parameters.amount_range.unwrap().max))
            })
            .cloned()
            .collect();
    
        let status = if data.is_empty() {
            "no data found"
        } else {
            "successful"
        }.to_string();
    
        Ok(QueryResponse { data, status })
    }

}

#[account]
pub struct EconomicData {
    pub node_id: Pubkey,           // The public key of the submitting node.
    pub invoice_data: InvoiceData, // Contains the HSNNumber, amount, quantity, and timestamp.
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, 
        seeds = [b"example".as_ref()], bump,
        space = 8 + 32 * 10)]
    pub state: Account<'info, NftState>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateNfts<'info> {
    #[account(mut)]
    pub state: Account<'info, NftState>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ShowNfts<'info> {
    #[account(mut)]
    pub state: Account<'info, NftState>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[account]
pub struct NftState {
    pub nfts: Vec<Pubkey>,
}

#[account]
#[derive(Default)]
pub struct StakerStats {
    stake_amount: u64,
    buy_amount: u64,
}

#[account]
#[derive(Default)]
pub struct AdminStats {
    stake_paused: bool,
    withdraw_paused: bool,
    bump: u8,
    stake_count: u64,
    lock_time: i64,
    stake_amount: u64,
    staker_amount: u32,
    buy_paused: bool,
    buy_amount: u64,
    buyer_count: u32,
}

impl AdminStats {
    pub const LEN: usize = 8 + 1 + 1 + 1 + 8 + 8 + 8 + 4 + 1 + 8 + 4;
}

#[account]
pub struct Node {
    pub node_id: Pubkey,
    pub is_valid: bool,
    pub total_transactions: u64,
    pub total_amount: u64,
    pub data: Vec<EconomicData>,
}

#[account]
pub struct NodeStatus {
    pub node_id: Pubkey,
    pub is_valid: bool,
    pub status: String,
}

#[account]
pub struct InvoiceData {
    pub hsn_number: String,
    pub amount: u64,
    pub quantity: u64,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct SubmitEconomicData<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + NodeAccount::BASE_SIZE,  // Calculate the size manually
        seeds = [b"DATAMESH_NODE", user.key.as_ref()],
        bump
    )]    
    pub node: Account<'info, NodeAccount>,  // NodeAccount is your custom struct for the account
    #[account(mut)]
    pub user: Signer<'info>,                // The user who is paying for the transaction
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ValidateNode<'info> {
    #[account(mut)]
    pub node: Account<'info, NodeAccount>,
    #[account(mut)]
    pub admin: Signer<'info>,                // The user who is paying for the transaction
}

#[derive(Accounts)]
pub struct QueryEconomicData<'info> {
    pub node: Account<'info, NodeAccount>,
}

#[derive(Accounts)]
pub struct GetNodeStats<'info> {
    pub node: Account<'info, NodeAccount>,
}

#[derive(Accounts)]
pub struct RemoveNode<'info> {
    #[account(mut)]
    pub node: Account<'info, NodeAccount>,
}

#[account]
pub struct NodeAccount {
    pub node_id: Pubkey,
    pub data: Vec<EconomicDataEntry>,
    pub active_since: i64,
    pub is_active: bool,
    pub total_rewards: u64,
}

impl NodeAccount {
    pub const BASE_SIZE: usize = 32 + 8 + 1 + 8; // node_id (Pubkey), active_since (i64), is_active (bool), total_rewards(u64)
}

#[account]
pub struct EconomicDataEntry {
    pub invoice_data: String,
    pub hsn_number: String,
    pub amount: u64,
    pub quantity: u32,
    pub timestamp: i64,
    pub signature: String,
    pub is_verified: bool
}

#[derive(Copy, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Range {
    pub min: u64,
    pub max: u64,
}

#[account]
pub struct QueryParameters {
    pub hsn_number: String,
    pub amount_range: Option<Range>,
}

#[account]
pub struct SubmitResponse {
    pub success: bool,
    pub transaction_hash: String,
}

#[account]
pub struct ValidateResponse {
    pub is_valid: bool,
    pub node_status: String,
}

#[account]
pub struct QueryResponse {
    pub data: Vec<EconomicDataEntry>,
    pub status: String,
}

#[account]
pub struct NodeStatsResponse {
    pub total_transactions: u32,
    pub total_amount: u64,
    pub active_since: i64,
}

#[account]
pub struct RemoveResponse {
    pub status: bool,
    pub message: String,
}