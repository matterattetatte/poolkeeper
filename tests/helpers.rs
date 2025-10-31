// In tests/helpers.rs or inline
trait MintExt {
    async fn new(
        banks_client: &mut BanksClient,
        payer: &Keypair,
    ) -> Result<Mint, TransportError>;
    async fn mint_to(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        to: &Pubkey,
        amount: u64,
    ) -> Result<(), TransportError>;
}

impl MintExt for Mint {
    async fn new(
        banks_client: &mut BanksClient,
        payer: &Keypair,
    ) -> Result<Self, TransportError> {
        let mint = Keypair::new();
        let rent = banks_client.get_rent().await?;
        let tx = Transaction::new_signed_with_payer(
            &[system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &Token::id(),
            ), spl_token::instruction::initialize_mint(
                &Token::id(),
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                9,
            )?],
            Some(&payer.pubkey()),
            &[payer, &mint],
            banks_client.get_latest_blockhash().await?,
        );
        banks_client.process_transaction(tx).await?;
        Ok(Mint { pubkey: mint.pubkey() })
    }

    async fn mint_to(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        to: &Pubkey,
        amount: u64,
    ) -> Result<(), TransportError> {
        let ix = spl_token::instruction::mint_to(
            &Token::id(),
            &self.pubkey,
            to,
            &payer.pubkey(),
            &[],
            amount,
        )?;
        let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
        tx.sign(&[payer], banks_client.get_latest_blockhash().await?);
        banks_client.process_transaction(tx).await
    }
}