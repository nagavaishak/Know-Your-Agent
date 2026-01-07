use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111"); // replace later

const MAX_PENALTY_PER_ACTION: u64 = 10;

#[program]
pub mod agent_registry {
    use super::*;

    pub fn register_agent(ctx: Context<RegisterAgent>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
    
        agent.agent_pubkey = ctx.accounts.user.key();
        agent.is_active = true;
    
        Ok(())
    }

    pub fn deactivate_agent(ctx: Context<DeactivateAgent>) -> Result<()>{
        let agent = &mut ctx.accounts.agent;

        require!(agent.agent_pubkey == ctx.accounts.user.key(), 
        CustomError::Unauthorized
        );

        require!(agent.is_active, 
            CustomError::AlreadyInactive
        );

        agent.is_active = false;

        Ok(())
    }

    pub fn perform_action(ctx: Context<PerformAction>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
    
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );
    
        agent.reputation = agent.reputation.checked_add(1).unwrap();
    
        Ok(())
    }
    

    pub fn reactivate_agent(ctx: Context<ReactivateAgent>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
    
        // Authority invariant
        require!(
            agent.agent_pubkey == ctx.accounts.user.key(),
            CustomError::Unauthorized
        );
    
        // State transition invariant
        require!(
            !agent.is_active,
            CustomError::AlreadyActive
        );
    
        agent.is_active = true;
    
        Ok(())
    }
    
    pub fn penalize_agent(ctx: Context<PenalizeAgent>, penalty_amount: u64,) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        let config = &ctx.accounts.config;
    
        // Authority invariant: only admin can penalize
        require!(
            config.admin_pubkey == ctx.accounts.admin.key(),
            CustomError::Unauthorized
        );
    
        // Agent must be active
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );
    
        // Reputation must be > 0
        require!(
            agent.reputation > 0,
            CustomError::AlreadyZero
        );
    
        // Penalty must be > 0 and within cap
        require!(
            penalty_amount > 0 && penalty_amount <= MAX_PENALTY_PER_ACTION,
            CustomError::PenaltyTooLarge
        );
    
        // Penalty must not exceed current reputation
        require!(
            penalty_amount <= agent.reputation,
            CustomError::ChoosePenalty
        );
    
        agent.reputation = agent.reputation.checked_sub(penalty_amount).unwrap();
    
        Ok(())
    }
    
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.base_price = 100;
        config.discount_threshold = 50;
        config.discount_percent = 50;
        config.min_reputation = 10;
    
        // Set the admin at initialization time
        config.admin_pubkey = ctx.accounts.user.key();
    
        Ok(())
    }
    
    pub fn get_price(ctx: Context<GetPrice>) -> Result<u64> {
        let agent = &ctx.accounts.agent;
        let config = &ctx.accounts.config;
    
        // Agent must be active
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );
    
        // Minimum Reputation gate
        require!(
            agent.reputation >= config.min_reputation,
            CustomError::LowReputation
        );
    
        let price = config.base_price;

        let price = if agent.reputation >= config.discount_threshold { In Rust, if returns a value, so you assign the whole if expression to a variable.
            base_price.checked_mul(100 - config.discount_percent as u64).unwrap().checked_div(100).unwrap()
        } else {
            base_price
        };
    
        Ok(price)
    }

    pub fn update_pricing_config(ctx: Context<UpdatePricingConfig>, base_price: u64, discount_threshold: u64, discount_percent: u8, min_reputation: u64,) -> Result<()> {

        let config = &mut ctx.accounts.config;

    // Authority check
    require!(
        config.admin_pubkey == ctx.accounts.admin.key(),
        CustomError::Unauthorized
    );

    // Validate inputs
    require!(
        discount_percent <= 100, //why we dint config.discount_percent because config. checks the existing stored value, not the new value the admin is trying to set
        CustomError::InvalidDiscount
    );

    // Update config
    config.base_price = base_price;
    config.discount_threshold = discount_threshold;
    config.discount_percent = discount_percent;
    config.min_reputation = min_reputation;
        
        Ok(())
    }

    pub fn perform_action_with_payment(ctx : Context<PerformActionWithPayment>) -> Result<()> {
        let agent = &mut ctx.accounts.agent;
        let config = &ctx.accounts.config;

        // Agent must be active
        require!(
            agent.is_active,
            CustomError::AgentInactive
        );

        // Minimum Reputation gate
        require!(
            agent.reputation >= config.min_reputation,
            CustomError::LowReputation
        );

        let base_price = config.base_price;

        let price = if agent.reputation >= config.discount_threshold { //In Rust, if returns a value, so you assign the whole if expression to a variable.
            base_price.checked_mul(100 - config.discount_percent as u64).unwrap().checked_div(100).unwrap()
        } else {
            base_price
        };

        // SOL transfer (CPI)
        let ix = anchor_lang::system_program::Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.treasury.to_account_info(),
        };

        anchor_lang::system_program::transfer(
            CpiContext::new(ctx.accounts.system_program.to_account_info(), ix),
            price,
        )?;
        
        //Perform the action
        agent.reputation = agent.reputation.checked_add(1).unwrap();

        Ok(())
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Agent is inactive")]
    AgentInactive,

    #[msg("Only the owner can modify the agent")]
    Unauthorized,

    #[msg("You cannot deactivate an already inactive agent")]
    AlreadyInactive,

    #[msg("Agent is already active")]
    AlreadyActive,

    #[msg("reputation is already 0")]
    AlreadyZero,

    #[msg("Penalty must be greater than 0 and less than or equal to reputation")]
    ChoosePenalty,

    #[msg("Penalty must be less than given MAX_PENALTY_PER_ACTION")]
    PenaltyTooLarge

    #[msg("Reputation too low to access service")]
    LowReputation,

    #[msg("Discount percent must be between 0 and 100")]
    InvalidDiscount,

}

#[account]
pub struct Agent {
    pub agent_pubkey: Pubkey,
    pub is_active: bool,
    pub reputation: u64,
}

#[account]
pub struct GlobalConfig {
    pub admin_pubkey: Pubkey,      // who controls config
    pub base_price: u64,            // normal price
    pub discount_threshold: u64,    // reputation needed for discount
    pub discount_percent: u8,       // % discount (0–100)
    pub min_reputation: u64,        // below this → blocked
}

#[account]
pub struct Treasury {} ///Empty becuase SOL balance is tracked by the runtime


#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"agent", user.key().as_ref()],
        bump,
        space = 8 + 32 + 1 + 8
    )]
    pub agent: Account<'info, Agent>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DeactivateAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", user.key().as_ref()],
        bump
    )]

    pub agent: Account<'info, Agent>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct PerformAction<'info> {
    #[account(
        mut,
        seeds = [b"agent", user.key().as_ref()],
        bump
    )]

    pub agent: Account<'info, Agent>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReactivateAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", user.key().as_ref()],
        bump
    )]

    pub agent: Account<'info, Agent>,
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct PenalizeAgent<'info> {
    #[account(
        mut,
        seeds = [b"agent", agent.agent_pubkey.as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,

    pub admin: Signer<'info>,
}


#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"config"],
        bump,
        space = 8 + 32 + 8 + 8 + 1 + 8
    )]
    pub config: Account<'info, GlobalConfig>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    #[account(
        seeds = [b"agent", agent.agent_pubkey.as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePricingConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, GlobalConfig>,

    pub admin: Signer<'info>,
}

// Treasury PDA
#[derive(Accounts)]
pub struct PerformActionWithPayment<'info> {
    #[account(
        mut, //agent is mut → reputation updated
        seeds = [b"agent", agent.agent_pubkey().as_ref()],
        bump,
    )]
    pub agent: Account<'info, Agent>,

    #[account(
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, GlobalConfig>,

    #[account(
        mut, //treasury is mut → SOL added
        seeds = [b"treasury"],
        bump,
    )]
    pub treasury = Account<'info, Treasury>,

    #[account(mut)]
    pub user: Signer<'info>, ////user is mut → SOL deducted

    pub system_program: Program<'info, System>,
}