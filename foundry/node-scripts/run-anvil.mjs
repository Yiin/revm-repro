import { spawn } from "child_process";
import { exec } from "./utils/exec.mjs";

const WETH_CONTRACT_ADDRESS = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

async function main() {
  const [chain_account, buybot_account, x1000_account, ...snipers] =
    await fork_mainnet();
  await build_contracts();
  const counter = await deploy_counter(buybot_account);
  const x1000 = await deploy_x1000(x1000_account);

  console.log("\n--- Setup complete ---\n");
  console.log("Counter deployed to", counter);
  console.log("X1000 deployed to", x1000);
  console.log("X1000 deployed by (dev)", x1000_account.wallet);
  console.log("Snipers (pkeys.json):");
  console.log(
    JSON.stringify(
      {
        privateKeys: snipers.map((s) => s.private_key),
      },
      null,
      2
    )
  );

  for (const sniper of [...snipers, x1000_account]) {
    const balance = await exec(
      `cast call ${WETH_CONTRACT_ADDRESS} "balanceOf(address)(uint256)" ${sniper.wallet}`
    );
    console.log(`Sniper ${sniper.wallet} WETH balance:`, balance);
  }

  console.log(
    "\nForked mainnet is currently listening on 127.0.0.1:8545, press Ctrl+C to stop it and exit."
  );
}

main();

async function fork_mainnet() {
  console.log("Forking mainnet...");

  const command = spawn("anvil", [
    "--fork-url",
    "https://mainnet.infura.io/v3/7c6ea95cef2b4ee9bd8e56224560358f",
  ]);

  let output = "";
  let wallets;

  command.stdout.on("data", (data) => {
    output += data.toString();

    // Only parse output when "Listening on " is encountered
    if (!wallets && output.includes("Listening on ")) {
      const accounts = output.match(/"0x[a-fA-F0-9]{40}"\s\((.*?)\sETH\)/g);
      const keys = output.match(/0x[a-fA-F0-9]{64}/g);

      if (accounts && keys) {
        wallets = [];
        for (let i = 0; i < accounts.length; i++) {
          let wallet = accounts[i]
            .match(/"0x[a-fA-F0-9]{40}"/)[0]
            .replace(/"/g, "");
          let private_key = keys[i];
          wallets.push({ wallet, private_key });
        }
      }
      // console.log(output);
    } else {
      // console.log(data.toString());
    }
  });

  // Ensure command's output error is handled
  command.stderr.on("data", (data) => {
    console.error(data.toString());
  });

  // Handle process close event
  command.on("close", (code) => {
    console.log(`anvil process exited with code ${code}`);
    process.exit(code);
  });

  // Listen for a process exit event and kill the child process
  process.on("exit", () => {
    command.kill();
  });

  // Listen for a SIGINT signal and kill the child process
  process.on("SIGINT", () => {
    console.log("Bye!");
    command.kill();
    process.exit(); // It's important to manually terminate the Node.js process
  });

  // Wait until wallets information is obtained
  while (!wallets) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  return wallets;
}

async function build_contracts() {
  console.log("Building contracts...");
  await exec(`forge build`);
}

async function deploy_counter(account) {
  console.log("Deploying Counter...");
  const output = await exec(
    `forge create --private-key ${account.private_key} src/Counter.sol:Counter`
  );

  const deployedTo = output.match(/Deployed to: (0x[a-fA-F0-9]{40})/);
  const address = deployedTo[1];

  return address;
}

async function deploy_x1000(account) {
  console.log("Deploying X1000...");
  const output = await exec(
    `forge create --private-key ${account.private_key} src/X1000.sol:X1000`
  );
  const deployedTo = output.match(/Deployed to: (0x[a-fA-F0-9]{40})/);
  const address = deployedTo[1];
  // console.log("Transfering X1000 tokens back to contract...");
  // await exec(`cast send ${address} --unlocked --from ${account.wallet} "transfer(address,uint256)(bool)" ${address} 1000000`)
  console.log("Transfering 1000 ETH to X1000...");
  await exec(
    `cast send --private-key ${account.private_key} ${address} --value=1000ether`
  );

  return address;
}
