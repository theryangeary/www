## I caused a security vulnerability today

I have been using docker swarm to deploy many services in my homelab. And I have not cared for the experience. I have effectively created a monorepo in which the process to deploy any one of the n services runs in O(n) time. So I've started instituting workarounds that allow me to deploy _some_ things in O(1) time. And today that caused a vulnerability.

## The set up

I have one `docker-compose.yml` file with 10 services defined in it. Some of them have config files, which I use docker swarm's config management to provide. Compared to using volume mounts, this allows the config to live in the git repository on my development machine, get loaded from my development machine into the docker swarm manager node, and then be provided to the services. With the volume mounts, the config files would have to live on the manager node, and if I ever scaled up to more than one node, they would need to be provided to those nodes as well.

[There are a few ways of defining these configs](https://docs.docker.com/reference/compose-file/configs/) in the `docker-compose.yml`, but in the process of originally setting up my services on my homelab, I went with the easiest method and used `external`. I think this is also the only way to define them on my local machine and make them available on the manager node without syncing the files onto the manager node's filesystem as well.

This meant that I was creating the configs manually from the command line, like so:

```bash
ssh $MYHOST 'docker config create <config_name> <source_file>`
```

Once a config is attached to a service, you cannot change it, so I devised a system to deploy a new config in a blue/green deployment, where I have a script that updates `config_name_blue` if `config_name_green` is in use, and vice versa. This allowed me to define both color configs in the config section of my `docker-compose.yml` once, and never have to touch them again. The only thing left to do was update the running container to use the new config.

## And that's where things fell apart

So I created a script which would assign the new config to the service. When you assign a new config to a service, docker swarm creates a new container with the new config, replacing the old container (subject to your service's `deploy` definition). 

And that's the problem. The script made a new container with the new config, which went live, but did not update the `docker-compose.yml` to reflect which config was active. I later used a deployment of the entire `docker-compose.yml`, because docker swarm was acting a bit buggy or funky in a way that I have not yet figured out, my service was broken, and a [little traffic was trickling in from a hackernews post](https://news.ycombinator.com/item?id=45818854).

It doesn't help that I have absolutely no automated observability yet.

## What broke

I use caddy as a reverse proxy to route traffic from my cloudflare tunnel to my various services. My previous caddy config included a reverse proxy with no authentication to the subdomain of the service that I had linked on hackernews. The reason I had deployed a new caddy config was to add authentication for a particular admin path, which I was using to moderate user submitted content. Admittedly, I could have-and maybe should have-implemented some authorization at the application level, however, I'm trying to move fast and fight perfectionism (and more importantly, the stakes for the actual security are quite low here).

When I triggered a deploy of the full docker stack, my `docker-compose.yml` still named the color deploy of the old config, and so docker did not complain and deployed it. It was not until eight hours later that I happened to check that page and noticed that I was not asked to login beforehand.

## How can I prevent this from happening again?

As of now, I'm not entirely certain how I'm going to move forward to prevent this, but it seems like there are a few options.

1. I could delete the old config once it is no longer used. The downside of this is that it makes intentionally rolling back harder.
2. I could update my config deploy script to update the `docker-compose.yml` so that the service points to the correct config. This might work fine, but I'm a little disgruntled about how much manual scripting I have to do to work around docker swarm.
3. I could start generating my `docker-compose.yml` more programmatically. The process of generating the markup could include checking with the currently used config is.
4. I could go nuclear and ditch docker swarm, maybe go to kubernetes or even something else entirely. k8s seems like even more work, but also seems like it has a better ecosystem of tooling so this type of thing might not be an issue.

There are probably a lot of other options as well, but this is kind of where my head is at so far. I think more than likely we will be pursuing option three.
