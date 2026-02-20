# leben-regex

Innehåller en exempelversion av `grep`, kör med `cargo run --example grep 
-- <REGEX>` (input genom stdin).

Tillåten regex-syntax är `|` för alternativ, `()` för gruppering och `*` för Kleene-konstruktioner. Escape sequences är `\|`, `\*`, `\(`, `\)`, `\\`.

Använder biblioteket `parsable` för regex parsing, som jag utvecklade under 
compiler-läxan.
