# Commercial license and attribution

This distribution of Ignis uses the **Ignis Engine Commercial Source License
1.0**. Read [LICENSE](../LICENSE) for the binding terms. The source is available
for inspection, modification and permitted redistribution. This custom commercial
license is **not an OSI-approved open source license**.

## What you pay

- Free to start, with no engine subscription, per-seat fee or account requirement.
- Each Game has its own **USD 1,000,000 lifetime revenue exemption**, across stores
  and platforms. There is no annual reset or retroactive charge on exempt profit.
- After that threshold: **1% of eligible profit**, after documented eligible Game
  costs and unrecovered losses. There is no royalty when there is no eligible profit.
- Non-Game uses and independent output without Ignis at runtime do not incur a
  royalty under this license. Your game's independent source and content can stay
  proprietary; running a game does not make a player a royalty-paying developer.

## How profit is calculated

Keep exempt and post-threshold receipts separate. Recover unpaid-for development
costs and other eligible Game losses. In a quarter crossing the threshold, allocate
costs using reliable contemporaneous records, or proportionally to revenue when
those records do not support a reliable split. Do not deduct the same cost twice.
The full cost rules, payment credits and reporting terms are in LICENSE.

Once the threshold is crossed:

```text
Eligible Profit = max(0, cumulative post-threshold revenue
                        - cumulative eligible post-threshold costs
                        - unrecovered loss at the threshold)
Cumulative Royalty = Eligible Profit * 1%
Current Payment = max(0, Cumulative Royalty - royalties already paid)
```

Examples, with no opening loss unless stated:

| Case | Calculation | Royalty |
| --- | --- | --- |
| Lifetime Game revenue USD 900,000 | Still below the revenue threshold | USD 0 |
| Quarter wholly after the threshold: revenue USD 300,000, eligible costs USD 250,000 | Profit USD 50,000 | USD 500 |
| Same revenue/costs, plus USD 80,000 unrecovered opening loss | Balance minus USD 30,000; no eligible profit | USD 0 |
| Crossing quarter: prior revenue USD 900,000; quarter revenue USD 200,000 and costs USD 120,000, split proportionally; no prior losses | USD 100,000 exempt and USD 100,000 post-threshold; USD 60,000 costs allocated to each; eligible profit USD 40,000 | USD 400 |

Negative balances carry forward for that Game. Later losses can create a credit
against future royalties already paid; LICENSE does not generally require cash
refunds of those credits. This distinction matters when forecasting costs.

## Credit

Retain LICENSE, NOTICE, relevant upstream notices and modification notices.
The required project attribution is:

> Ignis Engine — created by Guilherme Oliveira (zerobytes). https://github.com/zerobytes/ignis

Use legal notices, documentation or another readable permitted location. No
compulsory splash screen or advertising endorsement is required.

## Reporting and historical licenses

Reports and payments become due quarterly after the threshold, with a 60-day
window. Keep financial reports private. See [reporting guidance](royalty-reporting.md)
and [third-party notices](../THIRD_PARTY_NOTICES.md).

This distribution's license does **not** revoke rights granted for earlier
Apache-2.0 copies. A new repository, name or Git history does not change that.
