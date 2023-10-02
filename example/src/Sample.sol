// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/console.sol";

contract Sample {
    uint256 public number;
    string public name;
    address public owner;

    constructor(string memory _name) payable {
        name = _name;
        owner = msg.sender;
    }

    fallback() external payable {}

    function returnSender() external view returns (address) {
        return msg.sender;
    }

    function pureFunction() external pure returns (uint256) {
        return 1;
    }

    function pureFunction2() external pure returns (uint256) {
        return 1;
    }

    function returnBalance() external view returns (uint256) {
        return msg.sender.balance;
    }

    function payableFunction() external payable returns (uint256) {
        return msg.value;
    }

    function require0xbbbb() external returns (bool) {
        require(msg.sender == address(0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB));
        return true;
    }

    function keccak(bytes calldata b) external returns (bytes32) {
        console.logBytes(b);

        return keccak256(b);
    }

    function getMixedTuple() external view returns (uint64 namedRetVal, string memory, uint8) {
        string memory lit = "number mod two";
        return (uint64(number), lit, uint8(number % 2));
    }

    function intAndUint(int256 a, uint256 b) external view returns (int256, uint256) {
        return (a, b);
    }

    function maxUint(uint256 num) external returns (uint256) {
        return type(uint256).max;
    }

    function setNumber(uint256 newNumber) public {
        number = newNumber;
    }

    function setName(string calldata _name) public {
        name = _name;
    }

    function increment() public returns (uint256) {
        if (number == 3) {
            revert("number is 3");
        }
        number++;
        return number;
    }
}
