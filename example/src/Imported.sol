pragma solidity ^0.8.0;

contract Imported {
    struct S {
        uint256 a;
        bytes b;
        bytes32 c;
    }

    uint256 public setMe;

    constructor(uint256 _set) {
        setMe = _set;
    }

    function abcdefg(uint256) public pure returns (S memory) {
        return S(1, abi.encode(8888), bytes32(abi.encode(0x69)));
    }
}
