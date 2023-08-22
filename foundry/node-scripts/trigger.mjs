import { exec } from "./utils/exec.mjs";

const SNIPER_WALLET_ADDRESS = "0x90F79bf6EB2c4f870365E785982E1f101E93b906";
const DEV_WALLET_PRIVATE_KEY =
  "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a";
const X1000_CONTRACT_ADDRESS = "0xEb63D671653489B91E653c52a018B63D5095223B";

async function main() {
  // call approve on X1000 contract
  const res = await exec(
    `cast send ${X1000_CONTRACT_ADDRESS} "approve(address,uint256)(bool)" ${SNIPER_WALLET_ADDRESS} 10000000000000000000000000 --private-key ${DEV_WALLET_PRIVATE_KEY}`
  );

  console.log(res);
}

main();
