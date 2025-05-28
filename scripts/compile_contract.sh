#!/bin/bash

# Check if solc is installed
if ! command -v solc &> /dev/null; then
    echo "Error: solc is not installed. Please install it first."
    exit 1
fi

# Create build directory if it doesn't exist
mkdir -p build

# Compile the contract
echo "Compiling NewsVerification.sol..."
solc --abi --bin --overwrite -o build contracts/NewsVerification.sol

# Generate the contract ABI JSON
echo "Generating contract ABI..."
cat build/NewsVerification.abi | jq '.' > build/NewsVerification.json

echo "Contract compilation completed successfully!" 