FROM quay.io/pypa/manylinux2014_x86_64:latest	

#ENV PATH /opt/python/cp35-cp35m/bin/:/opt/python/cp36-cp36m/bin/:/opt/python/cp37-cp37m/bin/:/opt/python/cp310-cp310m/:$PATH

ENV PATH /root/.cargo/bin:$PATH
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && rustup update

RUN yum update -y
RUN yum install -y  centos-release-scl llvm-toolset-7

RUN which python3.10
RUN python3.10 -m pip install maturin
RUN python3.10 -m pip show maturin
RUN python3.10 -m site
RUN python3.10 -m sysconfig

RUN mkdir /io
WORKDIR /io
RUN echo "#!/bin/bash" > ~/entry.sh
RUN echo "echo GOT ARGUMENTS \"\$@\"" >> /root/entry.sh
RUN echo "source scl_source enable llvm-toolset-7" >> /root/entry.sh
RUN echo "/opt/_internal/cpython-3.10.12/bin/maturin \$@" >> /root/entry.sh
RUN cat /root/entry.sh
RUN chmod +x /root/entry.sh
ENTRYPOINT ["/root/entry.sh"]
