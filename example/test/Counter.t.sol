// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import "forge-std/console.sol";
import {Sample} from "../src/Sample.sol";
// import {Counter2} from "../src/src2/sample.sol";

contract CounterTest is Test {
    Sample public sample;

    function setUp() public {
        sample = new Sample("");
        sample.setNumber(0);
    }

    //7f6020600052600d6020527f48656c6c6f2c20576f726c642100000000000000006000527460405260606000f30000000000000000000000000060205260336000f3
    // SAMPLE: Load initcode + return caller 6008600b343960076000f3 335f5260205ff3
    function test_LoadAddress() public {
        console.logBytes(address(0x5B9feA3fC4A69d0510E815632D1813096f75458B).code);
    }

    function test_Increment() public {
        sample.increment();
        vm.breakpoint("a");
        assertEq(sample.number(), 1);
    }

    function testFuzz_SetNumber(uint256 x) public {
        sample.setNumber(x);
        assertEq(sample.number(), x);
    }
}
