# Examples of grpcurl

Call endpoints with [grpcurl](https://github.com/fullstorydev/grpcurl).

```sh
$ grpcurl -plaintext \
  -d '{"name":"tester","password":"abc123456"}' \
  localhost:9000 sky.UserService/SignUp
{
  "userId": "{USER_ID}"
}
```

```sh
$ grpcurl -plaintext \
  -H "authorization:Bearer {USER_ID}" \
  -d '{"content":"Hello, gRPC!"}'\
  localhost:9000 sky.PostService/Post
{
  "postId": "{POST_ID}",
  "createTime": "{TIMESTAMP}"
}
```

```sh
$ grpcurl -plaintext \
  -H "authorization:Bearer {USER_ID}" \
  -d '{"pageSize":3,"orderBy":"id asc"}' \
  localhost:9000 sky.PostService/ListPosts
{
  "posts": [
    ...
  ],
  "totalSize": "..."
}
```
