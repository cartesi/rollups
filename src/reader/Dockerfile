FROM node:12

# Create app directory
ADD . /code

WORKDIR /code

# Install app dependencies
COPY package*.json ./

RUN yarn preinstall

RUN yarn install


CMD ["yarn", "start"]

COPY . .

EXPOSE 4000