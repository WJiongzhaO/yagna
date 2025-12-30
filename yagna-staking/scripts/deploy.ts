import { ethers } from "hardhat";

async function main() {
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with the account:", deployer.address);

  const Staking = await ethers.getContractFactory("StakingManager");
  const staking = await Staking.deploy();

  await staking.waitForDeployment();

  console.log("StakingManager deployed to:", await staking.getAddress());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
