## API

Simple API server which allows you to find out which service accounts in a given namespace are in use by which pod


## Usage

```bash
curl -X GET localhost:8080/kube-system
```

Will return a json object like

```json
{
    "service-account-1": [
        "pod-1"
    ],
    "service-account-2": [
        "pod-2"
    ]
}
```

## Notes

- When running, you need to have a kubeconfig/mounted service account token in the usual locations. 
- Don't use this. It's mostly an experiment related to rust and it's various libraries. This lacks things that you would need in production, including:
  - An auth system
  - Support for TLS
  - Dockerfile to build in a container 
  - K8s Yaml/helm to install this in a cluster
  - Request logging system
  - CI pipeline to build/version
- Even if you implemented all of that, you just get a server which does something that you can probably achieve with a kubectl command

