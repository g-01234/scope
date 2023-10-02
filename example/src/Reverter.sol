// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Reverter {
    uint256 public number;
    string public name;

    error CustomError();

    constructor(bool revertInDeploy) {
        if (revertInDeploy == true) {
            revert("reverted in deploy");
        }
    }

    function justRevert() external {
        revert("revert message :)");
    }

    function revertStatic() external view {
        revert("reverted :o)");
    }

    function revertWithCustom() external view {
        revert CustomError();
    }
}
