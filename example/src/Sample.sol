// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Sample {
    uint256 public number;
    string public name;
    address public owner;

    constructor(string memory _name) payable {
        name = _name;
        owner = msg.sender;
    }

    receive() external payable {
        return;
    }

    function returnBalance() external view returns (uint256) {
        return msg.sender.balance;
    }

    function payableFunction() external payable returns (uint256) {
        return msg.value;
    }

    function onlyOwner() external view returns (bool) {
        require(msg.sender == owner);
        return true;
    }

    function getMixedTuple() external view returns (uint64 namedRetVal, string memory, uint8) {
        string memory lit = "number mod two";
        return (uint64(number), lit, uint8(number % 2));
    }

    function intAndUint(int256 a, uint256 b) external view returns (int256, uint256) {
        return (a, b);
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
