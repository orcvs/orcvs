# Bang is explicit rather than Function case

Orcvs does not adopt Orca's convention that uppercase Operators run every Tick while lowercase Operators require a neighbouring Bang. Functions have one canonical spelling, and Functions whose effects require triggering accept an explicit Bang operand, making scheduling visible and composable within prefix Expressions even when Functions are passed or produced as values.
