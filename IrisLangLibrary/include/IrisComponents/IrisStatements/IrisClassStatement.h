#ifndef _H_IRISCLASSSTATEMENT_
#define _H_IRISCLASSSTATEMENT_
#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisExpression;
class IrisBlock;
class IrisClassStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pClassName = nullptr;
	IrisExpression* m_pSuperClassName = nullptr;
	IrisList<IrisExpression*>* m_pModules = nullptr;
	IrisList<IrisExpression*>* m_pInterfaces = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisClassStatement(IrisIdentifier* pClasssName, IrisExpression* pSuperClassName, IrisList<IrisExpression*>* pModules, IrisList<IrisExpression*>* pInterfaces, IrisBlock* pBlock);
	virtual ~IrisClassStatement();

	virtual bool Validate() override;
};

#endif