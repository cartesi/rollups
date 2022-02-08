import { use } from "chai";
import { solidity } from "ethereum-waffle";
import { deployDiamond } from "./utils";

use(solidity);

describe("Diamond", () => {
    it("deploy Diamond", async () => {
        const diamond = await deployDiamond({ debug: true });
        console.log();
    });
});
