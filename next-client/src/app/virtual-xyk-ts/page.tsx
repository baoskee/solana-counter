"use client";

import { useCallback, useMemo, useState } from "react";
import { Curve, execute_state_transition, sol_out, token_out } from "./lib";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";

const USD_PER_SOL = 100;

export default function VirtualXykPage() {
  const [curve, setCurve] = useState<Curve>({
    token_amount: BigInt(1e9 * LAMPORTS_PER_SOL), // 1 billion in lamports
    virtual_sol_amount: BigInt(50 * LAMPORTS_PER_SOL), // 5K market cap = 50 SOL 
    actual_sol_amount: BigInt(0),
    distributed_token_amount: BigInt(0),
  });
  const [buyAmount, setBuyAmount] = useState<number>(0);
  const [sellAmount, setSellAmount] = useState<number>(0);

  const onBuyClick = useCallback(() => {
    const state = execute_state_transition(
      { curve },
      { type: "buy", amount: BigInt(buyAmount * LAMPORTS_PER_SOL) }
    );

    setCurve(state.curve);
  }, [curve, buyAmount])

  const youReceiveBuy = useMemo(() =>
    Number(token_out(curve, BigInt(buyAmount * LAMPORTS_PER_SOL)))
    / LAMPORTS_PER_SOL
    , [curve, buyAmount])

  const youReceiveSell = useMemo(() =>
    Number(sol_out(curve, BigInt(sellAmount * LAMPORTS_PER_SOL)))
    / LAMPORTS_PER_SOL
    , [curve, sellAmount])

  const onSellClick = useCallback(() => {
    const state = execute_state_transition(
      { curve },
      { type: "sell", amount: BigInt(sellAmount * LAMPORTS_PER_SOL) }
    );

    setCurve(state.curve);
  }, [curve, sellAmount])

  return <div>
    <div className="flex flex-col gap-2">
      <h1>Virtual Xyk Demo</h1>
      <h2>Curve - all amount in display precision (not lamports)</h2>
      <p>Market cap: ${USD_PER_SOL * Number(curve.virtual_sol_amount + curve.actual_sol_amount) / LAMPORTS_PER_SOL}</p>
      <pre>{JSON.stringify(curve, (key, value) =>
        typeof value === 'bigint' ? (Number(value) / LAMPORTS_PER_SOL).toString() : value
        , 2)}</pre>
    </div>

    <div className="py-2 flex gap-2 items-center">
      <p>
        You pay in SOL:
      </p>
      <input type="number"
        className="px-4 bg-slate-600 rounded-lg py-2"
        value={buyAmount}
        onChange={(e) => setBuyAmount(Number(e.target.value))}
      />
      <button onClick={onBuyClick}>
        Buy
      </button>
      <p>You receive: {youReceiveBuy}</p>
    </div>
    <div className="py-2 flex gap-2 items-center">
      <p>
        You pay in token:
      </p>
      <input type="number"
        className="px-4 bg-slate-600 rounded-lg py-2"
        value={sellAmount}
        onChange={(e) => setSellAmount(Number(e.target.value))}
      />
      <button onClick={onSellClick}>
        Sell
      </button>
      <p>You receive: {youReceiveSell}</p>
    </div>

  </div>
}
