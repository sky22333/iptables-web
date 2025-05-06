FROM python:3.8-alpine

RUN apk update && \
    apk add --no-cache iptables net-tools && \
    rm -rf /var/cache/apk/*

COPY . .

RUN pip install --no-cache-dir -r requirements.txt && \
    mkdir -p /app/data

EXPOSE 888

CMD ["python3", "app.py"]