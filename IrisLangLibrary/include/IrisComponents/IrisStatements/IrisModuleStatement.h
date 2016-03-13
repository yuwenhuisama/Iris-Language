#ifndef _H_IRISMODULESTATEMENT_
#define _H_IRISMODULESTATEMENT_
#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisIdentifier;
class IrisBlock;
class IrisExpression;
class IrisModuleStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pModuleName = nullptr;
	IrisList<IrisExpression*>* m_pModules = nullptr;
	IrisList<IrisExpression*>* m_pInterfaces = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisModuleStatement(IrisIdentifier* pModuleName, IrisList<IrisExpression*>* pModules, IrisList<IrisExpression*>* pInterfaces, IrisBlock* pBlock);
	virtual ~IrisModuleStatement();
};

#endif