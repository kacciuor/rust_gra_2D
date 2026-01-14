Bevy Endless Runner 2D
Prosta gra zręcznościowa typu endless runner stworzona w silniku Bevy 0.17.3 (Rust) https://bevy.org/. 
Projekt demonstruje podstawy architektury ECS, zarządzanie stanami gry oraz proceduralne generowanie obiektów.

🕹️ Mechanika
Skok: SPACE

Cel: Unikaj przeszkód i zbieraj monety, aby zwiększać wynik.

Trudność: Gra przyspiesza wraz z czasem.

Życie: Masz 3 HP i system krótkiej nieśmiertelności po zderzeniu.

🚀 Funkcje
Stany gry: Menu startowe, rozgrywka oraz ekran Game Over.

Dynamiczny HUD: Statystyki wyświetlane w czasie rzeczywistym.

Proceduralność: Przeszkody spawnowane w stałych interwałach dystansu.

🛠️ Instalacja
Bash

git clone https://github.com/kacciuor/rust_gra_2D
cd nazwa-repo
cargo run --release
