"use client";

import { anchorProvider, connectAnchorWallet, getProvider, LOCAL_RPC_URL, signAndBroadcast } from "@/lib/util"
import { Connection, Keypair, PublicKey, SystemProgram, Transaction } from "@solana/web3.js";
import * as anchor from '@coral-xyz/anchor';
import { useCallback } from "react";
import { escrowIDL, Escrow } from "anchor-local";
import { useQuery } from "@tanstack/react-query";
import { ASSOCIATED_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

// Fill out your params here
const TOKEN_MINT_A = "5CiESJk1uYGZ82S4YhaC5YCjRbUX1J4Q6Xf2ZWmYC6g7"
const TOKEN_MINT_A_AMOUNT = 420
const TOKEN_MINT_B = "5CiESJk1uYGZ82S4YhaC5YCjRbUX1J4Q6Xf2ZWmYC6g7"
const TOKEN_MINT_B_AMOUNT = 99_234
const TAKER = "HNc5mQKb5X7Agsk866kxM1yLv6dVDTPJnuPSsanGhrFo"

const OFFER_ID = 1; // increment this to create new offers

export default function EscrowPage() {
  // to get OTC offer accounts, simply derive PDA from seed in program
  // or use Anchor
  const offerDetails = useQuery({
    queryKey: ["offerDetails", OFFER_ID],
    queryFn: async () => {
      try {
        const provider = await anchorProvider();
        const program = new anchor.Program<Escrow>(
          // @ts-ignore
          escrowIDL, provider);

        const [offerPda, _] = PublicKey.findProgramAddressSync(
          [
            Buffer.from("otc_offer"),
            provider.wallet.publicKey.toBuffer(), // maker seed 
            new anchor.BN(OFFER_ID).toArrayLike(Buffer, "le", 8)
          ],
          program.programId
        )

        const offerDetails = await program.account.otcOffer.fetch(offerPda)

        return {
          ...offerDetails,
          tokenAAmount: offerDetails.tokenAAmount.toNumber(),
          tokenBAmount: offerDetails.tokenBAmount.toNumber(),
          idSeed: offerDetails.idSeed.toNumber(),
        }
      } catch (e) {
        console.error(e)
      }
    }
  })

  const depositEscrow = useCallback(async () => {
    try {
      const provider = await anchorProvider();
      const program = new anchor.Program<Escrow>(
        // @ts-ignore
        escrowIDL, provider);
      const [makerTokenAAccount] = PublicKey.findProgramAddressSync(
        [
          provider.wallet.publicKey.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          new PublicKey(TOKEN_MINT_A).toBuffer()
        ],
        ASSOCIATED_PROGRAM_ID
      );
      const [otcOfferAddr] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("otc_offer"),
          provider.wallet.publicKey.toBuffer(), // maker seed 
          new anchor.BN(OFFER_ID).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      )
      const contractTokenAccount = PublicKey.findProgramAddressSync(
        [
          otcOfferAddr.toBuffer(),
          TOKEN_PROGRAM_ID.toBuffer(),
          new PublicKey(TOKEN_MINT_A).toBuffer()
        ],
        ASSOCIATED_PROGRAM_ID
      );
      const inst = await program.methods.initialize(
        // increment this to create new offers
        new anchor.BN(OFFER_ID), // u64 = 8 bytes
        new anchor.BN(TOKEN_MINT_A_AMOUNT), // u64 = 8 bytes
        new anchor.BN(TOKEN_MINT_B_AMOUNT), // u64 = 8 bytes
        new PublicKey(TAKER) // 32 bytes
      )
        .accounts({
          maker: provider.wallet.publicKey,
          tokenMintA: new anchor.web3.PublicKey(TOKEN_MINT_A),
          tokenMintB: new anchor.web3.PublicKey(TOKEN_MINT_B),
          // @ts-ignore
          contractTokenAccount: contractTokenAccount,
          // @ts-ignore
          tokenAccountMaker: makerTokenAAccount,
          otcOffer: otcOfferAddr,
          // @ts-ignore
          associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction();
      // @todo try running instruction() instead of rpc()
      // might be issue with encoding the arguments

      const taker = new PublicKey(TAKER)
      console.log(taker.toBuffer())
      console.log(taker.toBase58())
      console.log(inst.keys.map(k => k.pubkey.toBase58()))

      const tx = new Transaction().add(inst)
      const connection = new Connection(LOCAL_RPC_URL)
      const wallet = getProvider()
      await wallet.connect()
      const confirmation = await signAndBroadcast(connection, wallet, tx)

      console.log("confirmation", confirmation)

      // transaction signature
      console.log(inst);
      offerDetails.refetch()
    } catch (e) {
      console.error(e);
    }
  }, [offerDetails])

  const claimOffer = useCallback(async () => {
    if (!offerDetails.data) return
    try {
      const provider = await anchorProvider();
      const program = new anchor.Program<Escrow>(
        // @ts-ignore
        escrowIDL, provider);
      const [offerPda, _] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("otc_offer"),
          provider.wallet.publicKey.toBuffer(), // maker seed 
          new anchor.BN(OFFER_ID).toArrayLike(Buffer, "le", 8)
        ],
        program.programId
      )

      const res = await program.methods.claim()
        .accounts({
          maker: offerDetails.data.maker,
          taker: provider.wallet.publicKey,
          tokenMintA: offerDetails.data.tokenMintA,
          tokenMintB: offerDetails.data.tokenMintB, 
        })
        .rpc();

      console.log(res)
    } catch (e) {
      console.error(e)
    }
  }, [offerDetails.data])

  const escrowTokenABalance = useQuery({
    queryKey: ["escrowTokenABalance", TOKEN_MINT_A],
    queryFn: async () => {
      throw new Error("Not implemented")
    }
  })

  return <div>
    <div>
      <div>Token Mint A: {TOKEN_MINT_A}</div>
      <div>Token Mint A Amount: {TOKEN_MINT_A_AMOUNT}</div>
      <div>Token Mint B: {TOKEN_MINT_B}</div>
      <div>Token Mint B Amount: {TOKEN_MINT_B_AMOUNT}</div>
      <div>Taker: {TAKER}</div>
      <div>Offer ID: {OFFER_ID}</div>
    </div>
    <div>
      <div>Offer ID: {OFFER_ID}</div>
      <div>Offer Details:
        <pre className="text-xs">
          {offerDetails.data && JSON.stringify(offerDetails.data, null, 2)}
        </pre>
      </div>
    </div>
    <div className="flex flex-col gap-2 my-2">
      <button onClick={depositEscrow}>
        Deposit Escrow
      </button>
      <button onClick={claimOffer}>
        Claim offer with {offerDetails.data?.tokenBAmount} TokenB
      </button>
    </div>

  </div>
}