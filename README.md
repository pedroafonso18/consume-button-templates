# Consume Button Templates

A Rust-based microservice that processes WhatsApp webhook messages containing button interactions, specifically designed to handle "Vamos Simular!" button clicks and trigger automated responses through the Gupshup API.

## Overview

This service acts as a webhook consumer that:
- Listens to RabbitMQ messages containing WhatsApp webhook data
- Parses button interactions from WhatsApp messages
- Validates button text (specifically "Vamos Simular!")
- Sends automated responses via Gupshup API
- Logs interactions to a PostgreSQL database
- Handles FGTS (Fundo de Garantia do Tempo de Serviço) simulation workflows

## Features

- **WhatsApp Webhook Processing**: Parses incoming webhook data from WhatsApp Business API
- **Button Interaction Handling**: Specifically handles "Vamos Simular!" button clicks
- **Automated Responses**: Sends predefined messages through Gupshup API
- **Database Logging**: Stores all interactions in PostgreSQL for audit trails
- **Async Processing**: Built with Tokio for high-performance concurrent processing
- **Docker Support**: Containerized deployment with multi-stage builds
- **Error Handling**: Comprehensive error handling and logging

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   WhatsApp      │    │   RabbitMQ      │    │   This Service  │
│   Webhook       │───▶│   Queue         │───▶│   (Rust)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                          │
                                                          ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   PostgreSQL    │◀───│   Gupshup API   │◀───│   Processed     │
│   (Logs)        │    │   (Responses)   │    │   Messages      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Prerequisites

- Rust 1.85 or later
- PostgreSQL database
- RabbitMQ server
- Docker (for containerized deployment)

## Environment Variables

Create a `.env` file in the project root with the following variables:

```env
# Database URLs
DB_URL=postgresql://username:password@localhost:5432/main_db
DB_URL_LOGS=postgresql://username:password@localhost:5432/logs_db

# RabbitMQ
RABBIT_URL=amqp://username:password@localhost:5672

# API Keys
API_KEY_HUGGY=your_huggy_api_key
API_KEY_GUP=your_gupshup_api_key
API_KEY_HUGGY2=your_huggy2_api_key

# Logging
RUST_LOG=info
```

## Installation

### Local Development

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd consume-button-templates
   ```

2. **Install dependencies**:
   ```bash
   cargo build --release
   ```

3. **Set up environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Run the application**:
   ```bash
   cargo run --release
   ```

### Docker Deployment

1. **Build the Docker image**:
   ```bash
   docker build -t consume-button-templates .
   ```

2. **Run the container**:
   ```bash
   docker run -d \
     --name consume-button-templates \
     --env-file .env \
     consume-button-templates
   ```

## Database Schema

### Main Database
The service connects to a main database for fetching user data and connections.

### Logs Database
The service logs all interactions to a `button-answers` table:

```sql
CREATE TABLE "button-answers" (
    id SERIAL PRIMARY KEY,
    num VARCHAR NOT NULL,
    mensagem TEXT NOT NULL,
    resposta_cliente VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Message Flow

1. **Webhook Reception**: WhatsApp sends webhook data to your endpoint
2. **Queue Processing**: Webhook data is published to RabbitMQ
3. **Message Consumption**: This service consumes messages from RabbitMQ
4. **Button Validation**: Service checks if button text contains "Vamos"
5. **Response Generation**: If valid, sends automated response via Gupshup
6. **Database Logging**: All interactions are logged to PostgreSQL

## API Integration

### Gupshup API
The service integrates with Gupshup API to send WhatsApp messages:

- **Endpoint**: `https://api.gupshup.io/wa/api/v1/msg`
- **Method**: POST
- **Headers**: Content-Type, apikey, cache-control
- **Body**: Form data with channel, source, destination, message, and src.name

### WhatsApp Webhook Structure
The service expects webhook data in this format:

```json
{
  "entry": [
    {
      "changes": [
        {
          "field": "messages",
          "value": {
            "contacts": [...],
            "messages": [
              {
                "button": {
                  "payload": "Vamos Simular!",
                  "text": "Vamos Simular!"
                },
                "context": {
                  "from": "553840039592",
                  "gs_id": "...",
                  "id": "...",
                  "meta_msg_id": "..."
                },
                "from": "555180646622",
                "id": "...",
                "timestamp": "1752151670",
                "type": "button"
              }
            ],
            "messaging_product": "whatsapp",
            "metadata": {...}
          }
        }
      ],
      "id": "536438802876584"
    }
  ],
  "gs_app_id": "5e7432e3-0402-491d-8710-8a324998833b",
  "object": "whatsapp_business_account"
}
```

## Configuration

### Logging
The service uses structured logging with different levels:
- `RUST_LOG=debug` - Detailed debug information
- `RUST_LOG=info` - General information (default)
- `RUST_LOG=warn` - Warning messages only
- `RUST_LOG=error` - Error messages only

### Performance
- **Concurrent Processing**: Uses Tokio async runtime for high throughput
- **Connection Pooling**: Deadpool for PostgreSQL connection management
- **Message Acknowledgment**: Proper RabbitMQ message acknowledgment

## Monitoring

The service provides comprehensive logging for monitoring:

- **Application Start**: Logs when the service starts successfully
- **Message Processing**: Logs each webhook processing attempt
- **Button Validation**: Logs button text validation results
- **API Calls**: Logs Gupshup API request/response details
- **Database Operations**: Logs database fetch and insert operations
- **Errors**: Detailed error logging with context

## Troubleshooting

### Common Issues

1. **Database Connection Errors**:
   - Verify `DB_URL` and `DB_URL_LOGS` are correct
   - Check PostgreSQL server is running
   - Ensure database tables exist

2. **RabbitMQ Connection Issues**:
   - Verify `RABBIT_URL` is correct
   - Check RabbitMQ server is running
   - Ensure queue exists and is accessible

3. **API Key Errors**:
   - Verify all API keys are valid
   - Check API key permissions
   - Ensure API endpoints are accessible

4. **Docker Issues**:
   - Check if all environment variables are set
   - Verify network connectivity
   - Check container logs: `docker logs consume-button-templates`

### Log Analysis

Monitor these log patterns:
- `"Processing webhook..."` - Service is processing a new webhook
- `"Button text validation passed"` - Button text matches expected value
- `"Gupshup message sent successfully"` - API call succeeded
- `"Error processing webhook"` - Processing failed, check error details

## Development

### Project Structure
```
consume-button-templates/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config/              # Configuration management
│   ├── rabbit/              # RabbitMQ connection and consumer
│   ├── process/             # Webhook processing logic
│   ├── db/                  # Database operations
│   └── api/                 # External API integrations
├── Cargo.toml               # Rust dependencies
├── Dockerfile               # Container configuration
└── README.md               # This file
```

### Adding New Features

1. **New Button Types**: Modify `parse_webhook_data()` in `process.rs`
2. **New API Integrations**: Add new functions in `api.rs`
3. **Database Operations**: Add new functions in `db/` modules
4. **Configuration**: Add new environment variables in `config.rs`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The MIT License is a permissive license that allows for:
- Commercial use
- Modification
- Distribution
- Private use

The only requirement is that the license and copyright notice be included in all copies or substantial portions of the software.