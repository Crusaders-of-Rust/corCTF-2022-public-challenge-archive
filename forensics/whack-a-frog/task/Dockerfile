FROM kissgyorgy/redbean-onbuild as build

FROM scratch

COPY --from=build /redbean.com /
CMD ["/redbean.com", "-vv", "-p", "80"]

