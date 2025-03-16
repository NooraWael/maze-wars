import socket

def send_ping():
    message = r'{"type": "Ping"}'
    server_address = ('0.0.0.0', 2025)

    # Create a UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    try:
        # Send the message
        print(f'Sending "{message}" to {server_address}')
        sent = sock.sendto(message.encode(), server_address)

        # Receive response
        print('Waiting for response...')
        data, server = sock.recvfrom(4096)
        print(f'Received "{data.decode()}" from {server}')
    finally:
        sock.close()

if __name__ == "__main__":
    send_ping()