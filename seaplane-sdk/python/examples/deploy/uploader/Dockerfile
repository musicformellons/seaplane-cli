# Use a base image with Python installed
FROM python:3.10

# Set the working directory in the container
WORKDIR /app

# Copy the entrypoint script to the working directory
COPY entrypoint.sh .

# Set the entrypoint script as executable
RUN chmod +x entrypoint.sh

# Set the entrypoint command
ENTRYPOINT ["./entrypoint.sh"]