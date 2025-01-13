#include <algorithm>
#include <random>
#include <thread>
#include <vector>

const int LOOP = 255;
const float X_START = -2.0;
const float X_END = 2.0;
const float Y_START = -2.0;
const float Y_END = 2.0;
const int WIDTH = 16384;
const int HEIGHT = 16384;

int mandelbrot_calc_base(float x, float y)
{
    auto re = x;
    auto im = y;

    for (auto i = 0; i < LOOP; i++)
    {
        float re2 = re * re;
        float im2 = im * im;

        if (re2 + im2 > 4.0f)
            return i;

        im = 2 * re * im + y;
        re = re2 - im2 + x;
    }

    return LOOP;
}

void _naive_mandelbrot(int width, int height, int *plot, std::vector<int> rows)
{
    float dx = (X_END - X_START) / (width - 1);
    float dy = (Y_END - Y_START) / (height - 1);

    for (int row : rows)
    {
        for (int col = 0; col < width; col++)
        {
            float x = X_START + col * dx;
            float y = Y_END - row * dy;

            auto result = mandelbrot_calc_base(x, y);
            plot[row * width + col] = result;
        }
    }
}

void naive_mandelbrot(int width, int height, int *plot)
{
    int num_threads = 8;
    int n_rows = height / num_threads;
    std::vector<std::thread> thread_pool;
    std::vector<int> rows(height);
    std::iota(rows.begin(), rows.end(), 0);
    std::random_device rd;
    std::mt19937 rng(rd());
    std::shuffle(rows.begin(), rows.end(), rng);

    for (int i = 0; i < num_threads; i++)
    {
        auto start = rows.begin() + i * n_rows;
        auto end = start + n_rows;
        std::vector<int> workload(start, end);
        thread_pool.emplace_back([=]()
                                 { _naive_mandelbrot(width, height, plot, workload); });
    }

    for (std::thread &thread : thread_pool)
    {
        if (thread.joinable())
        {
            thread.join();
        }
    }
}

int main()
{
    std::vector<int> plot(WIDTH * HEIGHT);
    naive_mandelbrot(WIDTH, HEIGHT, plot.data());
}