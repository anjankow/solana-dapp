# From https://github.com/RimuTec/dev-container-typescript/blob/3888fe10ce92aa54237befc02ae1580ecb342a94/.devcontainer/Dockerfile

FROM node:22.10.0-bookworm
# Last version check on 21.10.2024.
# Check for new versions at https://nodejs.org/en/ or at https://hub.docker.com/_/node
# Note that if you want to use this container for a Create React Application (CRA)
# you may need to use a different version. Consult React documentation.
# [Manfred]


################################################################################
# Upgrade NPM
# node may come with an older version of npm. Ensure we have a specific npm.
RUN npm install -g npm@10.9.0
# Last version check on 21.10.2024.
# Newer versions might be available at https://www.npmjs.com/search?q=npm
# [Manfred]


################################################################################
# Install prerequisites
RUN apt-get update
RUN apt-get install -y \
    lsb-release \
    curl


################################################################################
# Install TypeScript
RUN npm install -g typescript@5.6.3
# Last version check on 21.10.2024.
# Newer versions might be available at https://www.npmjs.com/search?q=typescript
# [Manfred]
