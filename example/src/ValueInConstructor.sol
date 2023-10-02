pragma solidity 0.8.21;

contract ValueInConstructor {
    uint256 public value;

    constructor() payable {
        value = msg.value;
    }
}
