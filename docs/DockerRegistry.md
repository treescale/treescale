# About
Docker Registry is a storage system for storing Docker Container images. [Docker Distribution](https://github.com/docker/distribution) is a Docker's company implementation for that purposes, and they are offering cloud based Docker Container Building tools 
at [https://hub.docker.com/](https://hub.docker.com/). <br/>
But based on specific software stack which is used to build Docker Hub, the build process is slow and monitoring reporting is missing during real time container builds.<br/>
So we aimed to build our own Docker Registry using TreeScale as a Log transferring system, Monitoring system and Build server load balancer.

# Implementation
The project itself had 3 major parts
1. Docker Images (packaged container) storage
2. TreeScale embedded Docker Image builder - TreeScale handles outputs during build process and as an "output" event transferring to specific log analyzer attached to another TreeScale Node as an API
3. Build Queue on top of TreeScale - Using Queue System with TreeScale we found powerful way of Events load balancing and using that principle we made builders load balancing

![alt text](https://raw.githubusercontent.com/treescale/treescale/master/docs/img/treescale-docker-registry.png "TreeScale Docker Builders")

Using this structure we got about `3.5 times` faster builds, just because we didn't had any scheduler or cron job for extracting Queue, it is just `Event push and Event receive` principle!

# Production Cases
This system first time used by PicsArt.com as an internal build process for Android applications bundling, then we are extended this project to become build infrastructure for their all Docker based environment.<br/>
We tested as a public service at [https://treescale.com](https://treescale.com), and got about 120 users less than 3 months, and about 270 Docker image builds.<br />