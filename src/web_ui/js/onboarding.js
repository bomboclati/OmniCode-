class OnboardingWizard {
    constructor() {
        this.steps = [
            { title: 'Welcome', description: 'Welcome to OmniCode! Let\'s get you set up.' },
            { title: 'Your Role', description: 'What is your primary role?', input: 'role' },
            { title: 'Experience', description: 'What is your experience level?', input: 'level' },
            { title: 'Goals', description: 'What are your goals?', input: 'goals' },
            { title: 'Learning Path', description: 'Generating your personalized learning path...' },
            { title: 'Ready!', description: 'You\'re all set to use OmniCode!' },
        ];
        this.currentStep = 0;
        this.responses = {};
    }

    init() {
        this.render();
    }

    render() {
        const container = document.querySelector('.onboarding-wizard');
        if (!container) return;

        const step = this.steps[this.currentStep];
        const progress = ((this.currentStep + 1) / this.steps.length) * 100;

        container.innerHTML = `
            <div class="onboarding-progress">
                <div class="progress-bar" style="width: ${progress}%"></div>
                <span class="progress-text">Step ${this.currentStep + 1} of ${this.steps.length}</span>
            </div>
            <div class="onboarding-content">
                <h2>${step.title}</h2>
                <p>${step.description}</p>
                ${this.renderInput(step)}
            </div>
            <div class="onboarding-nav">
                ${this.currentStep > 0 ? '<button class="btn secondary" id="prev-step">← Back</button>' : ''}
                ${this.currentStep < this.steps.length - 1
                    ? '<button class="btn primary" id="next-step">Next →</button>'
                    : '<button class="btn primary" id="finish-onboarding">🚀 Get Started</button>'
                }
            </div>
        `;

        this.bindEvents();
    }

    renderInput(step) {
        if (!step.input) return '';

        switch (step.input) {
            case 'role':
                return `
                    <div class="onboarding-options">
                        <label class="option-card ${this.responses.role === 'backend' ? 'selected' : ''}">
                            <input type="radio" name="role" value="backend" ${this.responses.role === 'backend' ? 'checked' : ''}>
                            <span class="option-icon">⚙️</span>
                            <span>Backend Developer</span>
                        </label>
                        <label class="option-card ${this.responses.role === 'frontend' ? 'selected' : ''}">
                            <input type="radio" name="role" value="frontend" ${this.responses.role === 'frontend' ? 'checked' : ''}>
                            <span class="option-icon">🎨</span>
                            <span>Frontend Developer</span>
                        </label>
                        <label class="option-card ${this.responses.role === 'fullstack' ? 'selected' : ''}">
                            <input type="radio" name="role" value="fullstack" ${this.responses.role === 'fullstack' ? 'checked' : ''}>
                            <span class="option-icon">🌈</span>
                            <span>Full Stack Developer</span>
                        </label>
                        <label class="option-card ${this.responses.role === 'devops' ? 'selected' : ''}">
                            <input type="radio" name="role" value="devops" ${this.responses.role === 'devops' ? 'checked' : ''}>
                            <span class="option-icon">☁️</span>
                            <span>DevOps Engineer</span>
                        </label>
                    </div>
                `;
            case 'level':
                return `
                    <div class="onboarding-options">
                        <label class="option-card ${this.responses.level === 'beginner' ? 'selected' : ''}">
                            <input type="radio" name="level" value="beginner" ${this.responses.level === 'beginner' ? 'checked' : ''}>
                            <span>🌱 Beginner</span>
                        </label>
                        <label class="option-card ${this.responses.level === 'intermediate' ? 'selected' : ''}">
                            <input type="radio" name="level" value="intermediate" ${this.responses.level === 'intermediate' ? 'checked' : ''}>
                            <span>📈 Intermediate</span>
                        </label>
                        <label class="option-card ${this.responses.level === 'advanced' ? 'selected' : ''}">
                            <input type="radio" name="level" value="advanced" ${this.responses.level === 'advanced' ? 'checked' : ''}>
                            <span>🚀 Advanced</span>
                        </label>
                    </div>
                `;
            case 'goals':
                return `
                    <div class="onboarding-goals">
                        <textarea id="goals-input" placeholder="What do you want to build? (e.g., AI apps, web services, CLI tools)" rows="4">${this.responses.goals || ''}</textarea>
                    </div>
                `;
            default:
                return '';
        }
    }

    bindEvents() {
        const nextBtn = document.getElementById('next-step');
        const prevBtn = document.getElementById('prev-step');
        const finishBtn = document.getElementById('finish-onboarding');

        // Save radio selections
        document.querySelectorAll('input[type="radio"]').forEach(input => {
            input.addEventListener('change', () => {
                const name = input.name;
                this.responses[name] = input.value;
                document.querySelectorAll(`.option-card`).forEach(c => c.classList.remove('selected'));
                input.closest('.option-card')?.classList.add('selected');
            });
        });

        const goalsInput = document.getElementById('goals-input');
        if (goalsInput) {
            goalsInput.addEventListener('input', () => {
                this.responses.goals = goalsInput.value;
            });
        }

        if (nextBtn) {
            nextBtn.addEventListener('click', () => {
                this.currentStep++;
                this.render();
            });
        }

        if (prevBtn) {
            prevBtn.addEventListener('click', () => {
                this.currentStep--;
                this.render();
            });
        }

        if (finishBtn) {
            finishBtn.addEventListener('click', () => {
                this.complete();
            });
        }
    }

    complete() {
        const container = document.querySelector('.onboarding-wizard');
        if (container) {
            container.innerHTML = `
                <div class="onboarding-complete">
                    <h2>🚀 You're Ready!</h2>
                    <p>Your OmniCode experience is personalized.</p>
                    <p>Role: ${this.responses.role || 'Full Stack'}</p>
                    <p>Level: ${this.responses.level || 'Intermediate'}</p>
                    <button class="btn primary" onclick="app.switchScreen('chat')">Start Coding</button>
                </div>
            `;
        }
    }
}

const onboardingWizard = new OnboardingWizard();
export default onboardingWizard;
