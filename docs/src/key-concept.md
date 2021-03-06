核心概念
=========

API（Application Programming Interface）这个术语现在被赋予了太多的内涵，远远超出了它是一个Interface的本意。说起API，想到的往往是一个
URL地址，发送一个HTTP请求来传入特定的参数，在HTTP响应中返回特定的结果。
其实作为一个Interface，API代表的是一个标准，是一个服务的生产端和消费端之间的协议，双方可以按照这个标准独立开发，而无需关心接口背后具体的
实现。由于API这个概念被赋予了太多的含义，这里尽量避免使用API这个术语，而是使用服务，协议，上游服务，应用等名词来表达API的定义，提供方，和消费方。

JuAPI平台管理着**服务**，**应用**，**环境**，**用户**，**团队**这五个实体和之间的关系。

其中**环境**代表着一个API网关的实例或者集群，**服务**代表一个API的提供方，被部署在**环境**中，**应用**代表API的消费方，通过申请服务授权关联到
**服务**，每个**环境**会将相关的**服务**和**应用**的配置数据，转换为网关的配置信息，通过数据同步接口通知到APIHub网关。

**用户**和**团队**的实体用来组织JuAPI平台中开发者之间的协作，实现类似github的去中心化平等自由的管理结构。**环境**，**服务**，**应用**都
属于某个**用户**或某个**团队**，**用户**和**团队**的角色关系，决定了**用户**对于不同的**服务**，**应用**，**环境**的操作权限。用户在自己的
命名空间内可以使用平台的所有功能。


## 服务：Service

**服务**表示JuAPI系统内管理的一个具体的API服务，是JuAPI系统管理的核心对象。服务**Service**将一个API的协议**Schema**，上游服务**Upstream**，
和消费方**App**连接在一起，部署到某个运行环境**Env**中。

用户可以在自己的用户命名空间，或者有管理员身份的团队的命名空间下创建服务。

### Endpoint

服务在JuAPI系统中的访问入口，服务的访问入口由服务所在环境的网关访问入口和服务的监听路径组成：

    https://<env.gateway.endpoint>/<listen.path>/

### Schema

API的接口规范使用[OAS 3.0](https://swagger.io/specification/)（Open Api Specification）的格式来描述。系统通过这个接口定义文件提供
接口文档的渲染，Mock服务，客户端SDK生成，服务器端Stub代码生成等功能。

API Schema的设计，协作，和版本管理后续会在JuCraft API设计器项目中实现。

### Middleware

网关可以在转发请求的过程中对HTTP请求和响应进行一些操作，Middleware是实现这些功能的统一机制。网关提供的认证，限流，请求参数注入等功能都是通过
Middleware机制来实现的。

Middleware的设置分为两个级别，在服务上设置的Middleware是服务级的，，服务级的Middleware对整个服务有效，例如路径ACL可以限制某个上游服务只暴露
一部分接口；在SLA上设置的Middleware是应用级的，应用级的Middleware对使用服务的某个具体应用有效，例如可以设置针对应用的访问频次限制，或限制应用
可以访问的接口。

### SLA

服务对应用的访问授权是通过服务等级来统一设置的，用户申请使用服务时会选择特定的服务等级。

### Grant

Grant代表服务和应用之间的授权关系，记录了应用可以使用某个SLA级别的某个服务。


## 应用：App

应用代表了服务的消费端，系统为每个App分配app-key和app-secret作为调用服务是的身份认证标示，通过Grant关联到使用的服务。

用户可以在自己的用户命名空间，或者有管理员身份的团队的命名空间下创建应用。

## 环境：Env

环境代表着一个API网关实例或者集群。系统中的服务需要发布到某个环境，环境通过一个WebSocket接口将发布在这个环境下的服务设置（包括服务相关的上游服务，
中间件配置，应用设置），编码为API网关的配置格式，将服务的配置信息实时同步到API网关，实现对API网关的管理。

用户可以在有管理权限的团队的命名空间下创建环境，也就是说每个用户都可以创建一个自己的环境，部署一个自己管理的API网关，来管理自己的服务或者应用。

### Service Deployment

Service Deployment代表服务和环境之间的多对多关系。

### Upstream

Upstream上游服务代表网关背后提供具体服务的API实现，发送到网关的请求都会转发给上游实现。Upstream的相关设置中包含了有关上游的服务注册发现，服务
版本，负载均衡，断路器，灰度发布设置。

由于各个**环境**的网关有着不同的网络环境，一个API的上游服务配置是和具体部署的环境有关的。所以Upstream的在配置服务部署到环境时设置。


## 用户：User

用户JuAPI平台通过聚合OAuth2使用聚合账号登陆。私有部署版本可以整合企业内部的LDAP，OAuth2。


## 团队：Team

用户可以创建团队，邀请团队成员，在团队命名空间下创建服务、应用、环境。类似Github的org，团队和用户公用一个命名空间。

团队成员有owner，admin，member三种身份。

