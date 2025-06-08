# saguru (or dokoda or dokosa)

## Basic Example

```bash
curl https://api.openai.com/v1/embeddings \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": "The food was delicious and the waiter was very friendly.",
    "model": "text-embedding-3-small"
  }'
```

## Multiple Inputs Example

```bash
curl https://api.openai.com/v1/embeddings \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": [
      "The cat sat on the mat",
      "I love machine learning",
      "OpenAI creates powerful AI models"
    ],
    "model":  "text-embedding-3-small"
  }'
```

## Using Different Models

```bash
# Using text-embedding-3-large (more accurate but more expensive)
curl https://api.openai.com/v1/embeddings \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": "Your text to embed here",
    "model": "text-embedding-3-large"
  }'
```

## With Reduced Dimensions (for text-embedding-3 models)

```bash
curl https://api.openai.com/v1/embeddings \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": "Your text to embed here",
    "model": "text-embedding-3-small",
    "dimensions": 512
  }'
```

## Response Format

The API will return a JSON response like this:

```json
{
  "object": "list",
  "data": [
    {
      "object": "embedding",
      "embedding": [
        0.0023064255,
        -0.009327292,
        -0.0028842222,
        // ... (1536 dimensions for text-embedding-3-small)
      ],
      "index": 0
    }
  ],
  "model": "text-embedding-3-small",
  "usage": {
    "prompt_tokens": 8,
    "total_tokens": 8
  }
}
```
