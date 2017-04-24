**Open Source Version** [https://github.com/treescale/treescale/tree/old_docker_lvs](https://github.com/treescale/treescale/tree/old_docker_lvs)

# About 
[Docker Container](https://www.docker.com) is a modern type of Linux Containers which are allowing to separate Linux Kernel into multiple environments for software isolation. Unlike Virtual Machines, Docker containers 
are very lightweight and easy transferable across multiple servers and clusters. Based on ideal functionality Docker containers mostly used for fast deployments, in Continues Integrations and for Cloud based applications which requires distributed execution.

TreeScale system first time has been implemented for Docker Containers, because in cluster management, monitoring and orchestration Docker had a lot of incomplete things. <br />
TreeScale with [Go language](http://golang.org) implementation had tied integration with Docker Engine itself and using simple Event based TreeScale API you could handle all kind of custom events from your Docker containers
cluster or push events to individual containers or broadcast them across multiple clusters.

# Implementation
Because of Docker itself is written in [Go language](http://golang.org), we just implemented TreeScale functionality and integrated it with Docker Engine, so we got TreeScale and Docker Engine as a same application. <br />
This gave ability to control all Docker Containers actions specially detect and monitor container fails within few milliseconds (based on live TCP connections). <br />
In terms of scalability, with TreeScale Docker containers now started working independent from specific server, because using specific custom defined events it is possible to move containers across servers and transfer data events to newly started container without loosing any data.<br/>
![alt text](https://raw.githubusercontent.com/treescale/treescale/master/docs/img/tree-docker-diagram.png "TreeScale and Docker Engine")

# Why project is stopped ?
After having working version, and initial feedback from Docker Company and Community we decided to stop TreeScale development in this space, because there was 100s of Docker based Cluster management tools and in that space almost nobady cares about `performance` or faster fail detections. 
Also one of the biggest reason is that Docker company released his own cluster management, Google did same thing. <br />
But after testing TreeScale on this real world project we found that Tree based scalability is working great, so we decided to implement this system on different areas.