# Decoupling Dynamo Architecture & Supporting Multiple Target Runner Environments

## Overview

The goal is to decouple the core Dynamo SDK from any specific serving or orchestration backend and introduce a flexible, pluggable architecture that allows users to select from multiple runner environments (e.g., BentoML, Circus, Ray, Slurm, Docker Compose, etc.) for service deployment and execution.

This will be achieved by:
- Defining a clear, extensible interface for deployment targets and runners.
- Refactoring the SDK to depend only on these interfaces, not on any concrete implementation.
- Providing adapters for each supported environment.
- Allowing users to select or register their desired runner at runtime.


## Key Concepts

- **Dynamo Interfaces**: Abstract [interface](../src/dynamo/sdk/core/service/interface.py) service definition from its execution environment.

- **DeploymentTarget**: Concrete implementation of DeploymentTarget for a specific environment.
    For example: [Kubernetes](../src/dynamo/sdk/core/targets/kubernetes.py), [Local](../src/dynamo/sdk/core/targets/local.py), Ray,  Slurm etc)

- **Target Injection**: Used to inject the appropriate runner/target at runtime during dynamo lifecycle management. [set_deployment_target](../src/dynamo/sdk/cli/utils.py)

- **Deployment Artifacts**: A generic intermediate representation of artifacts would enable portability of user code.

## Link Syntax and Runtime Graph Management
The link syntax in Dynamo architecture defines service dependencies, forming a directed graph.

[RuntimeLinkedServices](../src/dynamo/sdk/core/service/interface.py) manages these links, tracking active and unused edges to optimize execution. At runtime, only a sub-graph defined by explicitly linked services is executed, optimizing resource usage by activating only necessary services. This approach allows for flexible deployments of llm architectures.


```mermaid
classDiagram
    class DynamoTransport {
        <<enumeration>>
        +DEFAULT
        +HTTP
    }

    class ServiceConfig {
        <<dictionary>>
    }

    class DynamoEndpointInterface {
        <<abstract>>
        +name() str
        +__call__(*args, **kwargs) Any
        +transports() List~DynamoTransport~
    }

    class ServiceInterface {
        <<abstract>>
        +name() str
        +config() ServiceConfig
        +inner() Type~T~
        +get_endpoints() Dict~str, DynamoEndpointInterface~
        +get_endpoint(name) DynamoEndpointInterface
        +list_endpoints() List~str~
        +link(next_service) ServiceInterface
        +remove_unused_edges(used_edges) None
        +inject_config() None
        +dependencies() Dict~str, DependencyInterface~
        +get_service_configs() Dict~str, ServiceConfig~
        +service_configs() List~ServiceConfig~
        +all_services() Dict~str, ServiceInterface~
        +get_dynamo_endpoints() Dict~str, DynamoEndpointInterface~
        +__call__() T
        +find_dependent_by_name(service_name) ServiceInterface
        +dynamo_address() tuple~str, str~
    }

    class LeaseConfig {
        +ttl: int
    }

    class DynamoConfig {
        +enabled: bool
        +name: Optional~str~
        +namespace: Optional~str~
        +custom_lease: Optional~LeaseConfig~
    }

    class DeploymentTarget {
        <<abstract>>
        +create_service(service_cls, config, dynamo_config, app) ServiceInterface
        +create_dependency(on) DependencyInterface
    }

    class DependencyInterface {
        <<abstract>>
        +on() Optional~ServiceInterface~
        +get(*args, **kwargs) Any
        +get_endpoint(name) Any
    }

    class RuntimeLinkedServices {
        +add(edge) None
        +remove_unused_edges() None
    }

    class K8sEndpoint {
        +bentoml_endpoint: Any
        +name: Optional~str~
        +transports: Optional~List~DynamoTransport~~
        +__call__(*args, **kwargs) Any
    }

    class K8sService {

    }

    class K8sDependency {
        +bentoml_dependency: BentoDependency
        +on_service: Optional~BentoMLService~
        +dynamo_client: None
        +runtime: None
        +on() Optional~ServiceInterface~
        +get(*args, **kwargs) Any
        +set_runtime(runtime) None
        +get_endpoint(name) Any
        +get_bentoml_dependency() BentoDependency
    }

    class K8sDeploymentTarget {
        +create_service(service_cls, config, dynamo_config, app) ServiceInterface
        +create_dependency(on) DependencyInterface
    }

    DynamoEndpointInterface <|-- K8sEndpoint
    ServiceInterface <|-- BentoMLService
    DependencyInterface <|-- K8sDependency
    DeploymentTarget <|-- K8sDeploymentTarget
    RuntimeLinkedServices --> ServiceInterface
    BentoMLService --> K8sEndpoint
    BentoMLService --> K8sDependency
    K8sDependency --> K8sEndpoint
    K8sDeploymentTarget --> BentoMLService
    K8sDeploymentTarget --> K8sDependency
```

