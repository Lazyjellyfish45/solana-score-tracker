# Score Tracker - Programa Solana en Anchor

Proyecto de certificacion en el ecosistema Solana.
Vertical elegida: **Gaming**.

Score Tracker es un programa on-chain donde cada jugador puede crear su perfil, registrar puntajes de cada partida, cambiar su nombre publico y eliminar su perfil cuando ya no lo necesite. El estado de cada jugador vive en una cuenta PDA derivada de su wallet, por lo que cada wallet solo puede tener un perfil y solo el dueno puede modificarlo.

El proyecto cumple los requisitos de la certificacion: esta desarrollado en Solana con Rust + Anchor, implementa **CRUD + PDA**, y la documentacion vive tanto en este README como en los comentarios de `programs/score-tracker/src/lib.rs`.

## Estado del despliegue

- **Red:** Devnet
- **Program ID:** `8ySasPZ2NAcSM1w4KPzFg3AFZdkhBx381FJPvKqQTT39`
- **Solana Explorer:** https://explorer.solana.com/address/8ySasPZ2NAcSM1w4KPzFg3AFZdkhBx381FJPvKqQTT39?cluster=devnet
- **Build & Deploy:** realizado desde [Solana Playground](https://beta.solpg.io). El ciclo CRUD completo (create -> read -> update score -> update username -> delete) fue ejecutado y verificado en devnet.

---

## Tabla de contenidos

1. Que hace el proyecto
2. Arquitectura on-chain
3. Estructura del repositorio
4. Requisitos previos
5. Build, test y despliegue
6. Como interactuar con el programa
7. Mapeo CRUD + PDA
8. Notas de seguridad

---

## 1. Que hace el proyecto

Score Tracker permite a cualquier wallet de Solana:

- Crear un perfil de jugador con un username (hasta 32 caracteres).
- Registrar partidas sumando puntos al score acumulado.
- Cambiar el username cuando quiera.
- Cerrar su perfil y recuperar el rent (los lamports usados para guardar la cuenta).

La lectura del perfil se hace directamente desde el cliente con `program.account.playerProfile.fetch(pda)`. Es el patron estandar en Anchor: no hace falta una instruccion on-chain solo para leer.

## 2. Arquitectura on-chain

Cada jugador tiene una cuenta PDA con la siguiente estructura:

```
PlayerProfile {
    authority:    Pubkey   // dueno del perfil
    username:     String   // hasta 32 chars
    score:        u64      // puntaje acumulado
    games_played: u64      // partidas jugadas
    bump:         u8       // bump del PDA
}
```

El PDA se deriva con las semillas:

```
seeds = [ b"profile", authority.key().as_ref() ]
```

Esto garantiza que cada wallet solo puede crear un unico perfil y que la direccion es deterministica: cualquiera puede recalcularla desde el cliente sin necesidad de almacenarla.

## 3. Estructura del repositorio

```
solana-score-tracker/
|-- Anchor.toml
|-- Cargo.toml                  (workspace)
|-- package.json
|-- tsconfig.json
|-- programs/
|   `-- score-tracker/
|       |-- Cargo.toml
|       |-- Xargo.toml
|       `-- src/
|           `-- lib.rs          (programa Anchor con CRUD + PDA)
|-- tests/
|   `-- score-tracker.ts        (tests del ciclo CRUD)
|-- migrations/
|   `-- deploy.ts
|-- .gitignore
`-- README.md
```

## 4. Requisitos previos

- Rust 1.75+ (`rustup`)
- Solana CLI 1.18+ (`solana --version`)
- Anchor 0.30.1 (`avm install 0.30.1 && avm use 0.30.1`)
- Node 18+ y Yarn (o npm)

Configurar la wallet local si aun no existe:

```bash
solana-keygen new
solana config set --url localhost
```

## 5. Build, test y despliegue

Instalar dependencias:

```bash
yarn install
```

Compilar el programa:

```bash
anchor build
```

Sincronizar el `program id` (importante la primera vez, reemplaza el placeholder de `Anchor.toml` y `lib.rs` con la pubkey real generada al compilar):

```bash
anchor keys sync
anchor build
```

Levantar la red local y correr los tests:

```bash
anchor test
```

Desplegar a devnet:

```bash
solana config set --url devnet
solana airdrop 2
anchor deploy --provider.cluster devnet
```

## 6. Como interactuar con el programa

Ejemplo minimo en TypeScript usando el IDL generado por Anchor:

```ts
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.ScoreTracker;

const [profilePda] = PublicKey.findProgramAddressSync(
  [Buffer.from("profile"), provider.wallet.publicKey.toBuffer()],
  program.programId
);

// CREATE
await program.methods
  .createPlayer("eber_dev")
  .accounts({ profile: profilePda, authority: provider.wallet.publicKey })
  .rpc();

// UPDATE - sumar puntos
await program.methods
  .submitScore(new anchor.BN(120))
  .accounts({ profile: profilePda, authority: provider.wallet.publicKey })
  .rpc();

// READ
const data = await program.account.playerProfile.fetch(profilePda);
console.log(data);

// UPDATE - renombrar
await program.methods
  .updateUsername("eber_pro")
  .accounts({ profile: profilePda, authority: provider.wallet.publicKey })
  .rpc();

// DELETE
await program.methods
  .deletePlayer()
  .accounts({ profile: profilePda, authority: provider.wallet.publicKey })
  .rpc();
```

## 7. Mapeo CRUD + PDA

| Operacion | Instruccion                | Detalle                                                    |
|-----------|----------------------------|------------------------------------------------------------|
| CREATE    | `create_player(username)`  | `init` del PDA `[b"profile", authority]`                   |
| READ      | (cliente) `fetch(pda)`     | Lectura directa de la cuenta on-chain                      |
| UPDATE    | `submit_score(points)`     | Suma puntos y games_played +1                              |
| UPDATE    | `update_username(name)`    | Cambia el nombre publico                                   |
| DELETE    | `delete_player()`          | `close = authority`, devuelve el rent al dueno             |

## 8. Notas de seguridad

- Todas las instrucciones de modificacion validan el dueno con `has_one = authority`.
- El PDA garantiza unicidad por wallet: no se pueden crear perfiles duplicados.
- Las sumas de score usan `checked_add` para evitar overflow.
- `username` esta acotado a 32 caracteres y no puede estar vacio.

---

## Autor

Eber - Proyecto de certificacion Solana (Rust + Anchor).
