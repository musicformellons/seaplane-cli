#!/bin/bash

# Set the project folder name
PROJECT_FOLDER="project"

# Remove the project folder if it already exists
rm -rf "${PROJECT_FOLDER}"

# Create the project folder
mkdir "${PROJECT_FOLDER}"

# Download the zip file from the provided URL and save it in the project folder
wget -q -O "${PROJECT_FOLDER}/project.zip" "$1"

# Extract the zip file in the project folder
unzip -j "${PROJECT_FOLDER}/project.zip" -d "${PROJECT_FOLDER}"

# Install the required dependencies
pip install -r "${PROJECT_FOLDER}/requirements.txt"

# Change to the project folder
cd "${PROJECT_FOLDER}"

# Run the run.py script
python demo.py