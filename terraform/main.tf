terraform {
  required_providers {
    digitalocean = {
      source  = "digitalocean/digitalocean"
      version = "~> 2.0"
    }
  }
}

variable "do_token" {
  description = "Digital Ocean API token"
  type        = string
  sensitive   = true
}

variable "domain" {
  description = "Your domain name (e.g., loaa.yourdomain.com)"
  type        = string
}

variable "ssh_key_fingerprint" {
  description = "SSH key fingerprint for droplet access"
  type        = string
}

variable "admin_password" {
  description = "Admin password for Loa'a web interface"
  type        = string
  sensitive   = true
}

provider "digitalocean" {
  token = var.do_token
}

# Create a new droplet
resource "digitalocean_droplet" "loaa" {
  image    = "docker-20-04"  # Ubuntu 20.04 with Docker pre-installed
  name     = "loaa-server"
  region   = "sfo3"          # San Francisco - change to your preferred region
  size     = "s-1vcpu-1gb"   # $6/month - smallest size
  ssh_keys = [var.ssh_key_fingerprint]

  # Install docker-compose and set up environment
  user_data = <<-EOF
    #!/bin/bash
    set -e

    # Update system
    apt-get update
    apt-get upgrade -y

    # Install docker-compose if not present
    if ! command -v docker-compose &> /dev/null; then
      curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
      chmod +x /usr/local/bin/docker-compose
    fi

    # Create app directory
    mkdir -p /opt/loaa

    # Set up firewall
    ufw allow 22/tcp    # SSH
    ufw allow 80/tcp    # HTTP
    ufw allow 443/tcp   # HTTPS
    ufw allow 3001/tcp  # MCP server
    ufw --force enable

    # Generate JWT secret and environment config
    echo "LOAA_JWT_SECRET=$(openssl rand -base64 32)" > /opt/loaa/.env
    echo "LOAA_BASE_URL=https://${var.domain}" >> /opt/loaa/.env
    echo "LOAA_INCLUDE_MCP=true" >> /opt/loaa/.env
    echo "LOAA_MCP_PORT=3001" >> /opt/loaa/.env
    echo "LOAA_ADMIN_PASSWORD=${var.admin_password}" >> /opt/loaa/.env
  EOF

  tags = ["loaa", "production"]
}

# Firewall
resource "digitalocean_firewall" "loaa" {
  name = "loaa-firewall"

  droplet_ids = [digitalocean_droplet.loaa.id]

  inbound_rule {
    protocol         = "tcp"
    port_range       = "22"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "80"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "443"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "3000"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  inbound_rule {
    protocol         = "tcp"
    port_range       = "3001"
    source_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "tcp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }

  outbound_rule {
    protocol              = "udp"
    port_range            = "1-65535"
    destination_addresses = ["0.0.0.0/0", "::/0"]
  }
}

output "droplet_ip" {
  value       = digitalocean_droplet.loaa.ipv4_address
  description = "The public IP address of the droplet"
}

output "dns_instructions" {
  value = <<-EOT
    Add these DNS records in Squarespace:

    A Record:
      Host: loaa (or your subdomain)
      Points to: ${digitalocean_droplet.loaa.ipv4_address}
      TTL: 3600

    Then wait 5-10 minutes for DNS propagation.
  EOT
  description = "DNS configuration instructions"
}

output "infrastructure_summary" {
  value = <<-EOT
    Infrastructure Summary
    =====================
    Droplet: ${digitalocean_droplet.loaa.name}
    IP: ${digitalocean_droplet.loaa.ipv4_address}
    Region: ${digitalocean_droplet.loaa.region}

    Storage:
    - Droplet disk: 25GB (ephemeral)
    - Database: SurrealDB Cloud (wss://lexun-06dddvr74druj5v1jmbhaoqo8g.aws-usw2.surreal.cloud)
    - Namespace: loaa

    Monthly cost: $6/month (droplet only)

    Your database is hosted on SurrealDB Cloud with namespace-level isolation.
  EOT
  description = "Summary of provisioned infrastructure"
}
